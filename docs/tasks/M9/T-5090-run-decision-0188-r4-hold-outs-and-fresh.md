---
id: T-5090
title: "Run decision 0188 R4 hold-outs and fresh sweeps for T-4975's fits and models (waived in decision 0351)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Each T-4975 fit and scored structural model has its decision 0188 R4 hold-out and fresh-sweep result recorded, and its FITTED range covers where it applies.

## Context
Decision 0351 waived R4 for T-4975 (owner ruling). Owed: BETA.INV end-seed cut (beta.rs), GAMMA.DIST huge-shape band (gamma.rs), IRR section H (financial_util.rs, `IRR_SECANT_STEPS`, `IRR_FLAT_DIV_ZERO_NPV`, `IRR_LAST_UPDATE_TOLERANCE`), addendum G's fallback, LINEST/LOGEST QR, the AS 109 loop, CHISQ Halley, F.TEST ln B, NORM.INV centre. Added by T-5089 (decision 0351 addendum): the AS 109 floor-stop residual-sign split and F.INV's underflow exception, the slope stop at the first estimate, `Unconverged` → `#NUM!`, the overflowing-estimate crossing, CHISQ Halley above the Q-to-P switch, and LINEST's divided scale, alpha diagonal and identity-fit exception.

Filed by T-4975's review (PR #4073).

## Acceptance criteria
- [ ] The goal holds, shown by tests or eval cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
