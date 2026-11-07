---
id: T-4841
title: "IRR error codes from the secant's terminal events (#NUM! vs #DIV/0!)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-4840]
---

## Goal
Each terminal event of IRR's secant maps to Excel's error code, so the `#DIV/0!`-for-`#NUM!` cells match without the 3E+226 proxy.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 5. Absorbs T-3900; retires `IRR_FLAT_DIV_ZERO_NPV`.

## Acceptance criteria
- [ ] The 44 `#DIV/0!` cells and t2657-pv-irr-div0's 22 re-scored; the rule in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
