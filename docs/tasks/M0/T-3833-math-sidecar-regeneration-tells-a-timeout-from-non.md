---
id: T-3833
title: "Math sidecar regeneration tells a timeout from non-convergence"
milestone: M0
lane: harness
status: todo
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
- [ ] A timeout and non-convergence are separate outcomes. A timeout never removes a value that the
      existing sidecar already holds: the existing value is kept, or the run fails and asks for a
      per-case retry. Non-convergence keeps today's behaviour.
- [ ] Unit tests cover both cases (a stubbed slow evaluation, and a real non-converging one).
- [ ] Gate A's reference math is unchanged for points that converge (no table diffs on an idle
      regeneration).
- [ ] `make check-fast` green; reviewers approve (excel-fidelity included: Gate A reference math).

## Out of scope
- Changing the dps ladder or the convergence criterion.
