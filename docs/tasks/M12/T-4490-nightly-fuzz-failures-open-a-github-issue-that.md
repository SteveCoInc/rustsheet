---
id: T-4490
title: "Nightly fuzz failures open a GitHub issue that the issue intake works, so no red fuzz run goes unseen"
milestone: M12
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-4048]
---

## Goal
A failed nightly fuzz run becomes a task without anyone watching the Actions page. The run opens
an issue, or comments on the open one for that target, and the issue intake routine works it as a
P0 directive-5 bug.

## Context
`bitcode_workbook` failed nightly on 2026-11-02 and 2026-11-03 with timeouts, and no task was
filed. The steward skill had a rule for red fuzz runs (decision 0035), but no step of the daily
routine read scheduled runs, and a scheduled run on `main` wakes no agent. T-4048 fixed the
finding itself.

The owner chose this route (an issue from the workflow, worked by the existing issue intake) over
a daily steward sweep. It reacts within minutes and needs no change to a stored routine.

## Acceptance criteria
- [x] **Workflow:** `fuzz.yml` has a `report` job that runs only after a failed scheduled run on
      `main`. It is the only job with `issues: write` and `actions: read`; the fuzz jobs keep
      `contents: read`.
- [x] **Issues:** one issue per failed target (`Nightly fuzz: <target> failed`, label
      `fuzz-failure`) with the run link and the saved inputs embedded. A target with an open issue
      gets a comment instead.
- [x] **Formatter:** `scripts/fuzz_issue.py` formats the issue and validates job and target names,
      with unit tests (`scripts/test_fuzz_issue.py`).
- [x] **Intake:** the issue-intake skill has a "Fuzz finding" kind: P0, bytes as data, exempt from
      the per-author limit, and only for issues authored by `github-actions[bot]`.
- [x] **Docs:** the steward skill and agent-workflow.md name the issue route, count timeouts and
      OOMs as findings, and keep a backstop for a run with no issue.
- [x] **Decisions:** decision 0302 is recorded, with a revision line on decision 0035.
- [ ] `make check` green; reviewers approve.

## Out of scope
- **Other scheduled workflows:** the same blind spot for `nightly.yml` (nightly eval).
- **Routine prompts:** none change. The issue intake routine already fires on `issue.opened`.

## Outcome
**Workflow.** `.github/workflows/fuzz.yml`'s new `report` job does the following:
- lists the run's jobs and keeps the failed `fuzz (<target>)` ones;
- downloads that run's `fuzz-crash-*` artifacts;
- creates the `fuzz-failure` label if it is missing;
- for each failed target, comments on the open issue with the same title, or opens one.

**Formatter.** `scripts/fuzz_issue.py`:
- prints the failed targets, the title and the body;
- embeds inputs smallest first while the base64 stays under 48,000 characters, and lists the rest
  by size and SHA-1;
- names the failure kind from libFuzzer's artifact prefix;
- adds the `cfg(fuzzing)` trailing-bytes note for `bitcode_workbook`.

**Tests.** Eight unit tests cover:
- job-name filtering, including hostile names;
- embedding and the size limit;
- the no-artifact body;
- the repeat-failure comment;
- name validation.

**Dry run.** The `report` step's script, extracted from the YAML and run against a stub `gh`,
commented on an existing issue for one target (night 2's real artifact embedded, SHA-1 matching
libFuzzer's name). It opened a new issue for another target that had no artifact. It cannot run on
GitHub before merge, because it only runs after a failed scheduled run on `main`.

**Process docs.**
- The issue-intake skill's "Fuzz finding" kind and its per-author exemption.
- The steward rule and the agent-workflow.md row now say: issue route, timeouts and OOMs count,
  steward backstop.
- Decision 0302, and a revision of 0035.

**Unverified until the first real failure.** It is not yet known whether the intake routine's
trigger fires for an issue opened with the workflow token. The steward's daily issue sweep is the
backstop either way (decision 0302).
