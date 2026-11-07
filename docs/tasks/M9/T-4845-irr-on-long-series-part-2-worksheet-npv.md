---
id: T-4845
title: "IRR on long series, part 2: worksheet NPV bits and guess injection along the stalled path"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4844]
---

## Goal
Excel's worksheet NPV bits at the replica's stall points on long series, and where Excel's path parts from the replica's, are known.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 4, items 3 and 4.

## Acceptance criteria
- [ ] Probes of `NPV(r, flows)` at stall points of 21+ flows and guess injection along the replica's path, filled.
- [ ] financial.md records whether NPV's arithmetic or the solver differs on long series.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine change (T-4840).
