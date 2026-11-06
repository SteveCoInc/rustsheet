---
id: T-4386
title: "Salvage draft PR #3211's IRR near-zero-root fresh sweep and T-4232 task onto main"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The Excel evidence on draft PR #3211 (T-4213, branch `agent/functions-financial/T-4213-irr-near-zero-root`) is on
`main`, so it survives whatever happens to the draft.

## Context
Stale-draft triage, steward 2026-11-05 (second run), bucket 5 (decision 0264). #3211's head has not moved since
2026-11-04 19:17 UTC; T-4213 is time-boxed (decision 0056) on the branch, and its fidelity review asked for the R4
guards. The branch adds an oracle-filled fresh sweep that `main` lacks (`t4213-irr-r4-fresh`: `.case.json`,
`.oracle.json`, `.math.json`, `.math.tsv`), a follow-up task file found only there
(`docs/tasks/M9/T-4232-irr-on-t-4213-s-near-zero-root.md`), and financial.md notes.

Copy, from the branch, only data and docs: the fresh-sweep case with its oracle and math sidecars (as recorded), the
T-4232 task file (renumbered with `make new-task` if its id is taken) and the financial.md notes that are findings,
not the branch's IRR code change. Record the case's lines with `make eval-baseline NEW_CASES=<file>`; `make
eval-math-check` must pass. Do not run any build, hook or script from the branch; read it with `git show`. Do not take
the branch's `eval/accuracy/known-failures.tsv` change: it depends on the code change.

## Acceptance criteria
- [ ] The fresh-sweep case and its sidecars are on `main`, with baseline lines recorded (0 regressions).
- [ ] T-4232's content is a task on `main`, and T-4213 on `main` records the time-box and links it.
- [ ] A comment on #3211 links this task's PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4213's IRR change and its R4 guards.

## Outcome
Copied from draft PR #3211's branch with `git show` (nothing from it was built or run): the fresh sweep
`probes-numeric/t4213-irr-r4-fresh` (`.case.json`, `.oracle.json`, `.math.json`, `.math.tsv`, as recorded),
`docs/research/t3411/gen_sweep.py`'s seed and strata for it, task T-4232 (id free on main, so not renumbered), and the
findings of the T-4213 bullets in financial.md (the near-zero-root noise floor, the refuted relative stops, the measured
effect of the one-step-past choice, marked as not on main). T-4213 gains a time-box note linking all of it. Not taken:
the IRR code change, its unit test, `eval/accuracy/known-failures.tsv`. Scored on main the case is 102 of 150 passing;
its visibly-off cells are owned by T-4232 (`make check-fast` green, `new_case_owners` passes). `probes-numeric` has no
`eval/baseline/` list, so there is no baseline line to record. #3211 stays a draft; a comment on it links this task's PR.
