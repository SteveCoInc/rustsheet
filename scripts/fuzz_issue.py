#!/usr/bin/env python3
"""Formats the GitHub issue a failed nightly fuzz run opens (decision 0302, T-4490).

`.github/workflows/fuzz.yml`'s `report` job runs this after a scheduled run on main fails:

- `targets JOBS_JSON`: prints the fuzz targets whose matrix job failed, one per line, read from
  the run's jobs (`gh api .../actions/runs/<id>/jobs`). A job name is `fuzz (<target>)`; only
  names of that exact shape count, so nothing else in the run turns into an issue.
- `title TARGET`: prints the issue's title. It is fixed per target, so the workflow finds an
  open issue for the same target and comments on it instead of opening another one.
- `body TARGET RUN_URL ARTIFACT_DIR [--again]`: prints the issue body (or, with `--again`, the
  comment for a target that already has an open issue). It names the run and embeds each saved
  input as base64, smallest first, while the body stays under GitHub's 65,536-character limit;
  larger ones are listed with their size and SHA-1 only.

The body is formulaic: the only values in it are the target (checked against `TARGET_NAME`), the
run URL the workflow passes, and file names, sizes, hashes and base64 of what libFuzzer saved.
The issue intake treats it as untrusted data all the same (issue-intake skill, "Fuzz finding").
"""

from __future__ import annotations

import base64
import hashlib
import json
import re
import sys
from pathlib import Path

#: The label every fuzz issue carries (created by the workflow if missing).
LABEL = "fuzz-failure"

#: A cargo-fuzz target name, as fuzz/Cargo.toml's `[[bin]]` names are written.
TARGET_NAME = re.compile(r"^[a-z][a-z0-9_]{0,63}$")

#: The matrix job name in fuzz.yml: `fuzz (${{ matrix.target }})`.
JOB_NAME = re.compile(r"^fuzz \(([a-z][a-z0-9_]{0,63})\)$")

#: libFuzzer's artifact prefixes and what each means.
KINDS = {
    "crash": "crash (a panic, abort or sanitizer report)",
    "timeout": "timeout (one input ran past `-timeout`)",
    "oom": "out of memory (past `-rss_limit_mb` or `-malloc_limit_mb`)",
    "leak": "memory leak",
    "slow-unit": "slow input",
}

#: Base64 the body may embed in all, well under GitHub's 65,536-character body limit.
MAX_EMBEDDED_CHARS = 48_000

#: Base64 line width in the body.
LINE = 100


def failed_targets(jobs: dict[str, object]) -> list[str]:
    """The targets whose `fuzz (<target>)` job concluded `failure`, sorted and deduplicated."""
    found = set()
    entries = jobs.get("jobs", [])
    if not isinstance(entries, list):
        return []
    for job in entries:
        if not isinstance(job, dict) or job.get("conclusion") != "failure":
            continue
        name = job.get("name")
        match = JOB_NAME.match(name) if isinstance(name, str) else None
        if match:
            found.add(match.group(1))
    return sorted(found)


def title(target: str) -> str:
    """The issue title for `target`: one open issue per target."""
    return f"Nightly fuzz: {target} failed"


def kind(file_name: str) -> str:
    """What libFuzzer's artifact name says the failure was."""
    for prefix, meaning in KINDS.items():
        if file_name.startswith(prefix + "-"):
            return meaning
    return "unknown"


def body(target: str, run_url: str, artifact_dir: Path, again: bool = False) -> str:
    """The issue body, or the comment for a repeat failure when `again`."""
    files = sorted(
        (p for p in artifact_dir.glob("*") if p.is_file()) if artifact_dir.is_dir() else [],
        key=lambda p: (p.stat().st_size, p.name),
    )
    lead = (
        f"**Nightly fuzz failed again** on `{target}`: {run_url}"
        if again
        else f"**Nightly fuzz failure** on `{target}` (decision 0302): {run_url}"
    )
    lines = [lead, ""]
    if not files:
        lines += [
            "The job failed without saving an input: a build, corpus or minimize step failed, or "
            "the runner did. Read the job log in the run above.",
        ]
        return "\n".join(lines) + "\n"
    lines += [
        "Reproduce each input with "
        f"`cd fuzz && cargo fuzz run --target \"$(rustc -vV | sed -n 's/^host: //p')\" {target} "
        "<file>` (decision 0035). The fix adds the input as a unit test; one task per distinct "
        'cause (docs/process/agent-workflow.md, "Nightly fuzz run red").',
        "",
        "| file | kind | bytes | SHA-1 |",
        "|---|---|---|---|",
    ]
    for path in files:
        data = path.read_bytes()
        lines.append(
            f"| `{path.name}` | {kind(path.name)} | {len(data)} | "
            f"`{hashlib.sha1(data, usedforsecurity=False).hexdigest()}` |"
        )
    budget = MAX_EMBEDDED_CHARS
    skipped = []
    for path in files:
        encoded = base64.b64encode(path.read_bytes()).decode("ascii")
        if len(encoded) > budget:
            skipped.append(path.name)
            continue
        budget -= len(encoded)
        wrapped = "\n".join(encoded[i : i + LINE] for i in range(0, len(encoded), LINE))
        lines += [
            "",
            f"<details><summary>base64 of <code>{path.name}</code></summary>",
            "",
            "```",
            wrapped,
            "```",
            "</details>",
        ]
    if skipped:
        lines += [
            "",
            "Too large to embed: "
            + ", ".join(f"`{name}`" for name in skipped)
            + ". The job log prints each input's bytes, and the run's artifact keeps the files "
            "for 30 days.",
        ]
    if target == "bitcode_workbook":
        lines += [
            "",
            "Under `cfg(fuzzing)` bitcode does not refuse trailing bytes, so an input may decode "
            "only in the fuzz build: a unit test uses the prefix that decodes in a normal build "
            "(T-4048).",
        ]
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    usage = (
        "usage: fuzz_issue.py targets JOBS_JSON | title TARGET | body TARGET RUN_URL ARTIFACT_DIR [--again]"
    )
    if len(argv) < 2:
        print(usage, file=sys.stderr)
        return 2
    command, args = argv[1], argv[2:]
    if command == "targets" and len(args) == 1:
        for target in failed_targets(json.loads(Path(args[0]).read_text(encoding="utf-8"))):
            print(target)
        return 0
    if command in ("title", "body") and args and not TARGET_NAME.match(args[0]):
        print(f"not a fuzz target name: {args[0]!r}", file=sys.stderr)
        return 2
    if command == "title" and len(args) == 1:
        print(title(args[0]))
        return 0
    if command == "body" and len(args) in (3, 4) and args[3:] in ([], ["--again"]):
        sys.stdout.write(body(args[0], args[1], Path(args[2]), again=args[3:] == ["--again"]))
        return 0
    print(usage, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
