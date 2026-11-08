#!/usr/bin/env python3
"""Catch up a PR that conflicts with main (the merge queue removed it, or GitHub reports a
conflict), when the conflict is only in the eval-gated known-failure lists (decision 0207).

  scripts/merge_catchup.py --pr N   # exit 0: pushed; 1: refused (nothing pushed); 2: error

In this checkout, which must have no local changes, it does the following:
- merges `main` into the PR's branch without committing;
- refuses any conflict outside the lists CI's eval gate checks (`checked_baselines`: the
  gated suites the eval job runs without a fetch, the visible-parity lock and the usage-tier lock);
- takes `main`'s side of each conflicted known-failure list, resolves a conflicted visible-parity
  lock three-way per case from the merge base (`visible_lock.merge_visible`, decision 0219), and
  runs `make eval-baseline NEW_CASES=<the case files the PR adds>`, which re-records the lists,
  refuses regressions (decision 0093) and records visible-parity lines only for those cases
  (0219), and the usage-tier lock (decision 0355 item 6): main's side, re-recorded;
- refuses if the merged usage-tier lock has a tier unmet that main has met (T-5096);
- refuses if the re-record changed any other file, such as quality/evidence-floor.json;
- commits everything as one merge commit and pushes it to the PR branch (never forced);
- returns to the checkout's original branch or commit.

When CI is green on the pushed head, the PR is enqueued again without a re-review (work-loop step
6.3, decision 0247): the push is a merge of `main` whose only difference from the plain merge is
the content of those lists. The re-record builds and runs the PR's merged tree with this session's
credentials; forks are refused, so that tree is an agent branch of this repository.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from collections.abc import Callable, Sequence
from pathlib import Path

import gh_client
import visible_lock
from gh_client import field_of, text_of
from git_ops import BASE_BRANCH, REMOTE, ROOT, GitError, git, merge_tree
from suite_meta import suites_on

BASELINE_DIR = "eval/baseline/"
# `main` is fetched here, for the same reason as the PR head below.
BASE_REF = f"refs/merge-catchup/{BASE_BRANCH}"
RERECORD = ("make", "-s", "eval-baseline")
# The usage-tier lock (decision 0355 item 6, `gate_tiers::Lock`): one `<tier>\t<met|unmet>` line per
# tier, written only by `rustsheet-eval update-baseline`.
USAGE_GATES = f"{BASELINE_DIR}usage-gates.txt"
# The PR head is fetched here, not into a remote-tracking ref a single-branch clone may lack.
HEAD_REF = "refs/merge-catchup/head"


class Refused(Exception):
    """The PR cannot be caught up here; nothing was pushed."""


def outside_lists(paths: Sequence[str], lists: set[str]) -> list[str]:
    """The conflicted `paths` the catch-up cannot resolve: those not in `lists` (`resolvable_lists`)."""
    return [p for p in paths if p not in lists]


def fetch_base(root: Path) -> None:
    """Fetches `main` into `BASE_REF`, first turning a shallow clone into a full one (T-3499,
    decision 0201): cloud sessions clone shallow, and a merge base cannot be trusted there."""
    shallow = git("rev-parse", "--is-shallow-repository", root=root).stdout.strip() == "true"
    deepen = ("--unshallow",) if shallow else ()
    git("fetch", "-q", "--no-tags", *deepen, REMOTE, f"+refs/heads/{BASE_BRANCH}:{BASE_REF}", root=root)


def checked_baselines(root: Path) -> set[str]:
    """The lists CI's eval gate checks against the engine, on the `main` that `fetch_base`
    fetched: `eval/baseline/<suite>.txt` of every suite with `gate = true` and no `[fetch]` (the
    eval job fetches nothing), the visible-parity lock (decision 0219) and the usage-tier lock
    (decision 0355 item 6, T-5096). Fetched suites (poi, corpus-bench) and ungated ones are out."""
    suites = suites_on(root, BASE_REF)
    # CI names each list after the suite's `name`.
    locks = {visible_lock.VISIBLE_PARITY, USAGE_GATES}
    return locks | {f"{BASELINE_DIR}{s.name}.txt" for s in suites if s.gated and not s.fetched}


def tier_states(text: str) -> dict[str, str]:
    """The usage-tier lock's `tier -> met|unmet`; comments, blank and malformed lines are skipped."""
    states: dict[str, str] = {}
    for line in text.splitlines():
        parts = line.split("\t")
        if line.startswith("#") or len(parts) != 2:
            continue
        states[parts[0]] = parts[1]
    return states


def tier_flips(head: str, main: str) -> list[str]:
    """The tiers `main`'s usage-tier lock has met that `head`'s lock has unmet (or lacks): a catch-up
    never records that, since only `update-baseline --allow-regressions` may (decision 0355 item 6)."""
    merged = tier_states(head)
    return sorted(t for t, state in tier_states(main).items() if state == "met" and merged.get(t) != "met")


def conflicts_outside(root: Path, head: str, main: str) -> list[str] | None:
    """The one "does the merge conflict only in the lists?" check, before any merge is made (`make
    renumber`): the paths where merging `main` into `head` conflicts outside `resolvable_lists`;
    None when the merge cannot be computed. Needs `fetch_base` first."""
    merged = merge_tree(root, head, main)
    return None if merged is None else outside_lists(merged[1], resolvable_lists(root))


def resolvable_lists(root: Path) -> set[str]:
    """The paths whose conflicts the catch-up resolves, on the `main` that `fetch_base` fetched:
    the eval-gated lists (`checked_baselines`) and, when `main` has deleted it, the retired json
    lock a PR from before decision 0219 modified. `make renumber` uses the same set."""
    retired = not visible_lock.in_commit(root, BASE_REF, visible_lock.RETIRED_LOCK)
    return checked_baselines(root) | ({visible_lock.RETIRED_LOCK} if retired else set())


def rerecord(root: Path, new_cases: Path) -> int:
    """`make eval-baseline` in `root`: re-records eval/baseline/ and refuses regressions. Only the
    cases `new_cases` lists (the PR's added case files) may get a new visible-parity line or a
    higher one (`update-baseline --new-cases`, decision 0219)."""
    command = (*RERECORD, f"NEW_CASES={new_cases}")
    return subprocess.run(command, cwd=root, stdin=subprocess.DEVNULL, check=False).returncode


# What `catch_up` merges `main` into: given the checkout, the PR head and `main` (both fetched), the
# commit to check out before the merge and the merge commit's message. `make renumber` passes one
# that renumbers the PR's colliding ids first (scripts/merge_renumber.py, decision 0224).
Rewrite = Callable[[Path, str, str], tuple[str, str]]


def plain(root: Path, head: str, main: str) -> tuple[str, str]:
    """The catch-up's own `Rewrite`: the PR head itself."""
    return head, f"Merge {BASE_BRANCH} (make catch-up, decision 0207)"


def catch_up(
    root: Path,
    branch: str,
    record: Callable[[Path, Path], int] = rerecord,
    rewrite: Rewrite = plain,
    record_clean: bool = True,
) -> str:
    """Merges `main` into `branch` as the module docstring says, as one merge commit, and pushes
    it; returns the pushed head. Raises Refused when it cannot (nothing pushed). The checkout's branch, or detached HEAD, is restored either way. `rewrite` gives the
    commit the merge starts from (the merge commit's parents are still the PR head and `main`);
    without `record_clean`, the lists are re-recorded only when one conflicted."""
    branch = gh_client.branch_name(branch)
    if git("status", "--porcelain", "--untracked-files=no", root=root).stdout.strip():
        raise Refused("the checkout has local changes")
    on = git("symbolic-ref", "-q", "--short", "HEAD", root=root, check=False).stdout.strip()
    start = on or git("rev-parse", "HEAD", root=root).stdout.strip()
    fetch_base(root)
    main = BASE_REF
    git("fetch", "-q", "--no-tags", REMOTE, f"+refs/heads/{branch}:{HEAD_REF}", root=root)
    lists = resolvable_lists(root)
    head, main_sha = (git("rev-parse", ref, root=root).stdout.strip() for ref in (HEAD_REF, main))
    try:
        if git("merge-base", "--is-ancestor", main, HEAD_REF, root=root, check=False).returncode == 0:
            raise Refused(f"{branch} already contains {BASE_BRANCH}")
        start_at, message = rewrite(root, head, main_sha)
        git("checkout", "-q", "--detach", start_at, root=root)
        git("merge", "-q", "--no-ff", "--no-commit", main, root=root, check=False)
        if git("rev-parse", "-q", "--verify", "MERGE_HEAD", root=root, check=False).returncode:
            raise Refused(f"git could not start merging {BASE_BRANCH} into {branch}")
        conflicted = git("diff", "--name-only", "--diff-filter=U", root=root).stdout.split()
        outside = outside_lists(conflicted, lists)
        if outside:
            raise Refused(f"conflicts outside the eval-gated baseline lists: {', '.join(outside)}")
        for path in conflicted:
            if path == visible_lock.RETIRED_LOCK:
                git("rm", "-q", "--", path, root=root)
                continue
            if path == visible_lock.VISIBLE_PARITY:
                # main's side alone would drop the PR's lines, and either side's higher count would
                # undo main's lowerings: merge each case three-way (decision 0219). Stage 1 is the
                # merge base (absent when both sides added the file), 2 the PR, 3 main.
                base, pr, main_side = (git("show", f":{n}:{path}", root=root, check=False).stdout for n in (1, 2, 3))
                (root / path).write_text(visible_lock.merge_visible(base, pr, main_side), encoding="utf-8")
            else:
                git("checkout", "--theirs", "--", path, root=root)
            git("add", "--", path, root=root)
        # The PR's own case files, from before the merge: after it, the tree's baseline lists the
        # PR's cases too, so only this list tells them from main's (decision 0219).
        added = visible_lock.added_case_files(root, main, HEAD_REF)
        if conflicted or record_clean:
            with tempfile.TemporaryDirectory() as tmp:
                listed = Path(tmp) / "new-cases.txt"
                listed.write_text("".join(f"{p}\n" for p in added), encoding="utf-8")
                if record(root, listed) != 0:
                    raise Refused("make eval-baseline refused (a regression, or the eval did not run)")
        # The visible-parity check, before anything is pushed: a line above main's for one of
        # main's cases (a PR's sanctioned raise) needs a merge by hand (decision 0219).
        lock_path = root / visible_lock.VISIBLE_PARITY
        lock = lock_path.read_text(encoding="utf-8") if lock_path.exists() else ""
        main_lock = git("show", f"{main}:{visible_lock.VISIBLE_PARITY}", root=root, check=False).stdout
        raised = visible_lock.visible_rises(lock, main_lock, visible_lock.case_on(root, main))
        if raised:
            raise Refused(f"{visible_lock.VISIBLE_PARITY} has a line above main's ({raised[0]}): merge by hand")
        # The usage-tier check: the re-record refuses met -> unmet already; this holds the pushed tree
        # to it whatever the re-record did (decision 0355 item 6).
        tiers_path = root / USAGE_GATES
        tiers = tiers_path.read_text(encoding="utf-8") if tiers_path.exists() else ""
        flipped = tier_flips(tiers, git("show", f"{main}:{USAGE_GATES}", root=root, check=False).stdout)
        if flipped:
            raise Refused(f"{USAGE_GATES} has {', '.join(flipped)} unmet where {BASE_BRANCH} has it met")
        recorded = git("diff", "--name-only", root=root).stdout.split()
        others = sorted(p for p in recorded if p not in lists)
        if others:
            raise Refused(f"the re-record changed files outside the eval-gated lists: {', '.join(others)}")
        if recorded:
            git("add", "--", *recorded, root=root)
        # One merge commit of the PR head and main: main merged, lists re-recorded (and, from
        # `make renumber`, ids renumbered). A push the branch moved under is refused, not forced.
        tree = git("write-tree", root=root).stdout.strip()
        pushed = git("commit-tree", tree, "-p", head, "-p", main_sha, "-m", message, root=root).stdout.strip()
        git("push", "-q", REMOTE, f"{pushed}:refs/heads/{branch}", root=root)
        return pushed
    finally:
        git("merge", "--abort", root=root, check=False)
        git("checkout", "-q", "-f", *([] if on else ["--detach"]), start, root=root, check=False)


def pr_branch(client: gh_client.Client, pr: int) -> str:
    """The branch of PR `pr`; Refused unless the PR is open, from this repository and based on
    `main` (forks are refused: the tools build and push the PR's tree with this session's
    credentials)."""
    pull = client.pull(pr)
    if field_of(pull, "merged") is True or text_of(pull, "state") != "open":
        raise Refused(f"#{pr} is not open")
    if text_of(pull, "head", "repo", "full_name") != text_of(pull, "base", "repo", "full_name"):
        raise Refused(f"#{pr} comes from a fork")
    if text_of(pull, "base", "ref") != BASE_BRANCH:
        raise Refused(f"#{pr} is not based on {BASE_BRANCH}")
    return text_of(pull, "head", "ref")


def run_cli(argv: Sequence[str] | None, doc: str, name: str, verb: str, push: Callable[[Path, str], str]) -> int:
    """The command line of `make catch-up` and `make renumber`: `push(ROOT, branch)` on the PR
    `--pr` names (`pr_branch`); exit 0 pushed, 1 refused, 2 error."""
    parser = argparse.ArgumentParser(description=doc, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--pr", required=True, help="the PR's number")
    args = parser.parse_args(argv)
    client = gh_client.from_remote(ROOT)
    if client is None:
        print(f"{name}: origin is not a github.com remote", file=sys.stderr)
        return 2
    try:
        pr = gh_client.number(args.pr)
        head = push(ROOT, pr_branch(client, pr))
    except Refused as refusal:
        print(f"{name}: not {verb} #{args.pr}: {refusal}", file=sys.stderr)
        return 1
    except (gh_client.GhError, GitError) as error:
        print(f"{name}: {error}", file=sys.stderr)
        return 2
    print(f"#{pr}: pushed {head}; when CI is green, enqueue it again (work-loop step 6.3)")
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    return run_cli(argv, __doc__ or "", "merge_catchup", "catching up", catch_up)


if __name__ == "__main__":
    sys.exit(main())
