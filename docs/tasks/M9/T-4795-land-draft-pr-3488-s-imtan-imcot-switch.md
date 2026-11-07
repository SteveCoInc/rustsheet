---
id: T-4795
title: "Land draft PR #3488's IMTAN/IMCOT switch at 2y past 709 (T-4425) from a fresh branch"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-4425's bracketed IMTAN/IMCOT switch (Excel turns `#NUM!` into ±i where 2y > 709 or 2y < -708, past the 2^27 cut)
and its probe case are on `main`, reviewed, so draft PR #3488 can be closed as superseded.

## Context
Stale-draft triage, steward 2026-11-07, bucket 5 (decision 0264). Draft #3488 (branch
`agent/functions-engineering/T-4425-imtan-imcot-switch`, head 6f3fd5de, 2026-11-06 04:59 UTC) has T-4425 `status:
done` on the branch and green CI, but its review table was never run (all pending), so it is not ready (bucket 4).
`main` lacks its evidence: probes/t4425-imtan-imcot-switch (excel-graph, 192 cells), the engineering.md bracket, and
`tan_saturates` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs, which reuses IMCSCH's
evidenced `in_csch_negated_band` (T-3881/T-3912) rather than fitting a constant.

Take the case and oracle sidecar from the branch with `git show` (do not run its scripts), re-apply the code change
on a fresh branch from `main`, record the baseline with `make eval-baseline`, and run the review panel. Then
finish T-4425's bookkeeping on `main` (its Outcome is on the branch).

## Acceptance criteria
- [ ] probes/t4425-imtan-imcot-switch is on `main` with its oracle sidecar, and all 192 cells pass.
- [ ] IMTAN and IMCOT switch where the probe brackets it; a unit test pins both sides of the cut.
- [ ] engineering.md records the bracket; T-4425 is `status: done` with an Outcome on `main`.
- [ ] A comment on #3488 links this task's PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other IM* overflow residue (T-4721's notes).
