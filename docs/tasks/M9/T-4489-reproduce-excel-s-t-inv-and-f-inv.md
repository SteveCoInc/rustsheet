---
id: T-4489
title: "Reproduce Excel's T.INV and F.INV values at T-3544's five Gate A points (owner ruling 0301: Excel stays the target)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine returns Excel's own values at T-3544's five Gate A points, where Excel is more than G = 1e-4 off the math
and the owner ruled that Excel stays the target (decision 0301): accuracy/T.INV/s1 A91 (p 8.2E-131, df 66741) and
A96 (p 2.9E-116, df 91643), and accuracy/F.INV/s1 A164 (p 1.9E-93), A167 (`F.INV(4.36982683607437E-189,1,1698856)`,
Excel 3.85E-145) and A169 (p 1.2E-69).

## Context
- Decision 0301 item 3, T-3544. Excel's tiny-p quantiles come from its own search on its own cdf:
  - T.INV is SQRT(df·(1/y - 1)) of the rounded beta quantile y (T-3061; statistics.md, the Student-t quantile entry);
  - F.INV is a crossing of F.DIST's tails through Excel's BETA.INV (T-2555, T-3485).
- Today the engine returns the math there, which passes Gate A only through its math side.
- Decision 0056's time-box applies. If two attempts do not reproduce Excel, block the task as time-boxed: the points
  then keep passing through the math side, so nothing regresses.

## Acceptance criteria
- [ ] The five points match Excel within half a unit in the 15th significant digit (decision 0083), or the task is
      blocked as time-boxed.
- [ ] No other T.INV, F.INV or Gate A point moves out of the envelope. The decision 0267 R3 cells (T-3386, T-3337)
      keep the math.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The decision 0267 R3 cells (T-3386, T-3337).
