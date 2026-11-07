---
id: T-4838
title: "IRR engineered-event probes round 1: flat secant and pole crossing"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4836, T-4837]
---

## Goal
Excel's behaviour at a flat secant and at a pole crossing is observed directly, on inputs built to trigger each event at a chosen step, and the shortlist is cut by the result.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 3, items 1, 3 and 4.

## Acceptance criteria
- [ ] Probe cases with predictions committed before the fill, oracle-filled (excel-graph).
- [ ] financial.md records Excel's flat-secant and pole behaviour with the cells, and the models eliminated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cap (T-4839).
