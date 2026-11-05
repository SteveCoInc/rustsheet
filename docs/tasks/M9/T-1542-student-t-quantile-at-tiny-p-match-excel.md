---
id: T-1542
title: "Student-t quantile at tiny p: match Excel for df >= 99999 and to the last digit for small df"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
T.INV, T.INV.2T, TINV and CONFIDENCE.T at a tiny p match Excel to the last digit for every df, including df >= 99999 at p <= 1E-200 (T.INV(1E-300,99999) = -79.06351062982456).

## Context
T-1531 stopped the `statrs` panic with a tail asymptotic (`t_tail_quantile` in `functions/statistical/t_dist.rs`) but only 7 of 105 cells of probes-numeric `t1531-t-tiny-p` are bit-exact: most are about 1E-13 relative off, and df >= 99999 at p <= 1E-200 is wrong (37.17, the normal quantile, where Excel gives 79.06, because for df = 99999 the `statrs` cdf saturates in the bisection branch and for df = 1E5 the df >= 1E5 path uses a normal series; both paths need a fix). See statistics.md (Student-t quantile at a tiny probability). T.INV.2T with p in (1,2) is also `#NUM!` for us where Excel gives a negative number (cells A121-A128).

Cells from T-0967 (2026-11-05, `make eval` on origin/main 82b5cd14; T-0967 was their only owner): tiny p at a
small df (2 to 22), each first wrong at the 14th (decision 0267 Tier C, marked C) or 15th significant digit:
- gen-m9: 0620505280d2ea6e A152 `TINV(1E-200,8)`, 9e2225ead8a6a07b A40 `T.INV.2T(1E-200,8)`, 0bbab6206b5c9a07 A315
  `T.INV(1E-20,2)`, 139684e68fb2fc3b A90 `TINV(1E-100,9)`, 775d053de498b8c7 A238 `T.INV(1E-100,9)`, 1db18890916ee643
  A144 `TINV(1E-100,5)`, 41248d1cb153c66e A331 `TINV(1E-300,4)`, 607e85cfee83a689 A215 `T.INV(1E-100,8)`,
  7980ffb6095a340d A84 `TINV(1E-16,6)`, 8fec7b207f4dd753 A246 `CONFIDENCE.T(1E-300,1.005,23)`, 91e167cd8d7eb652 A350
  (C), 941c409532603d32 A384 (C) and eccd03df42e40006 A273 (C) `T.INV.2T(1E-200,7)`/`TINV(1E-200,7)`, 928d08e3e1eecb5f
  A147 and bcb1105c61a2649d A315 `T.INV(Data!..,4)` (Excel -7400.827819722825), 92d4fa29e5510f1e A67
  `TINV(1E-16,2.5)`, a5342a7e24fd803e A118 `TINV(1E-100,10)`, a63d1c7d6f96c56d A298 `TINV(1E-100,6)` (C),
  d910f000b60e2eac A212 `T.INV(1E-100,2)`, fd9fa22d212080d6 A11 `CONFIDENCE.T(1E-200,1.5E+10,22)`.
- At df >= 99999 and p <= 1E-200, where Excel is more than 1E-4 off the math, decision 0267 item 3 has the engine
  follow the math. Those cells are T-3386's R3 marks, not this task's.

## Acceptance criteria
- [ ] All 105 cells of `t1531-t-tiny-p` pass or the residual is a recorded DEVIATION.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
