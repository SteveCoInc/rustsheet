---
id: T-4842
title: "IRR last digits under the new driver: fallback, near-zero roots, visibly-off cells"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4840]
---

## Goal
IRR's returned digits match Excel's on the cells the new driver reaches with the right type.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 6. Absorbs T-2131, T-2928, T-4213, T-4232, T-4338.

## Acceptance criteria
- [ ] The visibly-off and ulp IRR cells re-scored under the new driver; the remaining ones owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
