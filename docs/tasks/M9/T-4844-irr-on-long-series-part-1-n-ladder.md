---
id: T-4844
title: "IRR on long series, part 1: n-ladder and fixed-point probes where Excel returns and the secant stalls"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-4838]
---

## Goal
Whether Excel's IRR changes regime with the number of flows, and whether its result on long series passes its own stop test, is known.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 4, items 1 and 2: 399 of the 434 cells no model gets are 21+ flows where Excel returns and the replica stalls (195) or caps (181).

## Acceptance criteria
- [ ] An n-ladder probe (10 residual series at n = 10..150 by trailing zeros and stretched annuities) and the
      fixed-point probe `IRR(flows, Excel's result)`, filled, with predictions.
- [ ] financial.md records the result and the hypotheses (H8, H9) it supports or refutes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NPV bits (T-4845).
