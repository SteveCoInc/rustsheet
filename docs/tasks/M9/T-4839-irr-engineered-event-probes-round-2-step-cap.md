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
