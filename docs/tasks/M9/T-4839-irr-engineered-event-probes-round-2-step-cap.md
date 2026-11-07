---
id: T-4839
title: "IRR engineered-event probes round 2: step cap and power-of-two scale ladders"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4838]
---

## Goal
Excel's behaviour at the step cap and the absolute-or-relative nature of its stop tests are pinned by cap-length runs and x2^k scale ladders.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 3, items 2 and 5; T-3117 asked for the scale ladders; T-3408 left 200..216 steps unclear.

## Acceptance criteria
- [ ] Probe cases (runs of 190..260 steps; x2^k ladders of stalled and residual-cut cells) with predictions, filled.
- [ ] financial.md records the cap rule and the tolerances' scaling.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Long series (T-4844).

## Owned cells (T-4844 review)
Excel's secant runs past the engine's 50-step cap on series longer than `IRR_LONG_RUN_RANGE`'s 54 flows (financial.md
"The IRR and XIRR solver caps"):
- probes/t4844-irr-guess-ladder P!A393:A401 and P!A432:A440: 61-flow annuities, converging at steps 54 to 195 in Excel
  and the uncapped replica, `#NUM!` in the engine.
- probes/t4844-irr-pad-ladder P!A32:A34: A51 padded to 592..928 flows, `#DIV/0!` in Excel at the step-62 underflow.

