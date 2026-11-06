---
id: T-4385
title: "Salvage draft PR #3105's CUMPRINC underflow probes and T-4059 task onto main"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The Excel evidence on draft PR #3105 (T-3141, branch `agent/functions-financial/T-3141-cumprinc-underflow-bits`) is
on `main`, so it survives whatever happens to the draft.

## Context
Stale-draft triage, steward 2026-11-05 (second run), bucket 5 (decision 0264). #3105's head has not moved since
2026-11-04 07:13 UTC. T-3141 is `status: blocked` on the branch on T-4059, a task file that exists only on that branch
(`docs/tasks/M9/T-4059-cumprinc-and-cumipmt-gate-a-tables-with-t.md`), so `main` has no record of the blocker; T-4110
(Gate A tables, annuity II and rates, CUMIPMT) is the nearest task on `main`. The branch also adds two oracle-filled
probe cases that `main` lacks: `fin-cumprinc-underflow-fresh` and `fin-cumprinc-underflow-type1` (`.case.json` and
`.oracle.json`), and T-3141 notes in financial.md.

Copy, from the branch, only data and docs: the two probe cases with their oracle sidecars (as recorded, never
refilled by hand), the T-4059 task file (renumbered with `make new-task` if its id is taken, merged into T-4110
instead if T-4110 already covers it) and the financial.md notes that are findings, not the branch's code change.
Record the new cases' lines with `make eval-baseline NEW_CASES=<file>`. Do not run any build, hook or script from the
branch; read it with `git show`. The engine change on the branch stays there (T-3141 owns it).

## Acceptance criteria
- [x] The two probe cases and their oracle sidecars are on `main`, with their baseline lines recorded (0 regressions).
- [x] The CUMPRINC Gate A table blocker is a task on `main` (T-4059's content, or T-4110 extended), and T-3141 on
      `main` names it in `depends_on`.
- [x] A comment on #3105 links this task's PR.
- [x] `make check` green; reviewers approve.

## Out of scope
- T-3141's engine change and its review.

## Outcome
From the T-3141 branch (read with `git show`, nothing run): the probe cases `fin-cumprinc-underflow-fresh` and
`fin-cumprinc-underflow-type1` with their oracle sidecars as recorded, T-4059's task file (its id is free on `main`,
kept as is; T-4110 does not cover CUMPRINC/CUMIPMT underflow sweeps), and the findings paragraph of financial.md's
CUMPRINC notes (the rule, the fit/hold-out record, the lost forms; not the "Ours since" paragraph or the Status line,
which describe the engine change T-3141 owns). T-3141's `depends_on` now names T-4059. Eval delta: probes +166
known-failure lines (the new cases score against the current engine; the engine fix stays on the branch), visible-parity
lock +1 line, every visibly-off cell owned (24 CUMIPMT cells of the fresh case now named in T-3141), 0 regressions. The comment on #3105 links this PR.
