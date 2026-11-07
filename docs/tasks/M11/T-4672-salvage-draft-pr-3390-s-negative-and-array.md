---
id: T-4672
title: "Salvage draft PR #3390's negative and array kept-subnormal save probe and saved_number flush onto main"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The work of draft PR #3390 (T-4355, branch `agent/engine/T-4355-negative-spilled-subnormal`) lands on `main` from a
fresh branch, so its oracle evidence survives whatever happens to the draft: the probe
`probes/subnormal-saved-negative-and-array` with its excel-graph sidecar, the `saved_number` flush of a kept
subnormal of either sign, the numbers.md entry, and the follow-up task the branch files (T-4372 there: the
array-context comparison `(EXP({-745,-708}))=0` that excel-graph answers TRUE).

## Context
Stale-draft triage, steward 2026-11-06 (evening run), bucket 5 (decision 0264). #3390's head (f8aeeaab) has not moved
since 2026-11-05 17:22 UTC. Its task is `status: done` on the branch and CI is green on the head, but its review
table is all `pending`, so it is not bucket 4. `main` has neither the probe nor the T-4372 task file (that id may
now collide: allocate a fresh one with `make new-task` and keep the branch's text).

Copy from the branch with `git show` (do not run its scripts): the probe case and oracle sidecar, the
`saved_number` change and its export test, the numbers.md entry, the T-4355 Outcome and the follow-up task. Record
the new probe's lines with `make eval-baseline` rather than copying the branch's baseline files.

## Acceptance criteria
- [ ] The probe, its oracle sidecar, the engine change with its test and the numbers.md entry are on `main`;
      T-4355 is `status: done` with the branch's Outcome; the follow-up task is filed under a fresh id.
- [ ] Eval: 0 regressions; the new probe's failing cells are owned by the follow-up task.
- [ ] A comment on #3390 links this task's PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel's save (M14).

## Outcome
Superseded: T-4355 (PR #3833) landed the negative and array kept-subnormal save probe (`probes/subnormal-saved-negative-and-array`) and the `saved_number` flush on main. Nothing left to salvage from draft PR #3390.
