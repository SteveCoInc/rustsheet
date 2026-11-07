---
id: T-3833
title: "Math sidecar regeneration tells a timeout from non-convergence"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Regenerating Gate A math sidecars gives the same result on a loaded machine as on an idle one: a
point that is only slow is never recorded as "not converged".

## Context
`tools/src/rustsheet_tools/accuracy/eval_math.py` has `NOT_CONVERGED = "not converged or too slow"`,
which lumps a wall-clock timeout together with real non-convergence. `tables.py` (around line 79,
"not converged, too slow, or no single value") leaves such a point out of the batch. In the
2026-11-02 cleanup, a full-suite regeneration on a busy machine turned 15 existing values into
skips; per-case regeneration restored them. The excel-fidelity reviewer of #2753 suggested this
follow-up.

## Acceptance criteria
- [x] A timeout and non-convergence are separate outcomes. A timeout never removes a value that the
      existing sidecar already holds: the existing value is kept, or the run fails and asks for a
      per-case retry. Non-convergence keeps today's behaviour.
- [x] Unit tests cover both cases (a stubbed slow evaluation, and a real non-converging one).
- [x] Gate A's reference math is unchanged for points that converge (no table diffs on an idle
      regeneration).
- [x] `make check-fast` green; reviewers approve (excel-fidelity included: Gate A reference math).

## Out of scope
- Changing the dps ladder or the convergence criterion.

## Outcome
- The behaviour was already delivered by T-3680 (merged before this task was picked): `tables.compute`
  no longer drops a point by wall clock. Its 600 s hang guard raises `PointTimeoutError`, which
  nothing catches, so `rustsheet-accuracy math` and the table writer fail the run instead of
  writing a skip; points are dropped (`DroppedError`, "not converged") only by deterministic
  budgets (dps 60/80 disagreement, `WORK_BUDGET` terms, mpmath's own `NoConvergence`/`maxterms`).
  The constant this task quoted ("not converged or too slow") is now "not converged".
- This PR adds the one missing test, on the eval-cell sidecar path (`eval_math.write_suite`):
  `test_a_timeout_fails_the_run_and_keeps_the_sidecar_a_drop_does_not` checks that a stubbed
  timeout fails `write_suite` and leaves the existing sidecar byte-identical, and that a
  non-convergence still lists the cell under `skipped`. The table path was already covered by
  `test_a_slow_point_stops_the_run_instead_of_being_dropped` and
  `test_points_that_do_not_converge_are_dropped_and_logged` (T-3680).
- Gate A reference math unchanged: no code change, no sidecar or table diff.
- Eval delta: none (test-only).
