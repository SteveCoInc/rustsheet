---
id: T-4833
title: "IRR deep research plan: diagnosis, feasibility checks and phased probe plan"
milestone: M9
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A research plan for raising IRR from 79.28% within 15 digits (the lowest function) to parity, grounded in measurements
rather than in the earlier tasks' hypotheses.

## Context
- IRR is 79.28% within 15 digits over 3,129 attributed cells (docs/progress/functions.md); 575 of its 727 off cells
  are the wrong result type, not digits (a full eval at d13a3dec).
- The secant in d is bit-exact on the fit set; the three events where the replica stops (flat secant, iterate past
  d = 1, step cap) have invented handling, patched by six range-limited FITTED constants in `compute_irr`.

## Acceptance criteria
- [x] docs/research/irr-2026-11-07/plan.md: what IRR is, its arguments and values, the failure breakdown, why the
      earlier fits plateaued, the leads, an eight-phase plan, targets and tasks.
- [x] docs/research/irr-2026-11-07/feasibility.py reproduces the plan's section 6 tables.
- [x] The plan's tasks are filed (T-4834 to T-4845).
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-4840).

## Outcome
The plan and its feasibility script landed with the task files of its section 10. No engine or eval change; no
baseline moved. Measured (feasibility.py, 2,555 literal IRR probe cells, outcome type): engine 2,000; RATE's fitted
driver transplanted onto IRR's secant 1,990 with no IRR-specific cuts (+118 cells the engine misses); the VB runtime's
failure handlers 1,874; VB's Financial.IRR verbatim is bit-exact on 3 of the 263 fit-set cells. The 434 cells no
candidate gets are 399 series of 21+ flows where Excel returns a root and the replica stalls or reaches its cap.
