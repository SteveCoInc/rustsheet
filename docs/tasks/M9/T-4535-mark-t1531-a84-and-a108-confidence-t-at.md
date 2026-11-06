---
id: T-4535
title: "Mark t1531 A84 and A108 (CONFIDENCE.T at p 1E-300) under decision 0267 item 3 once CONFIDENCE.T has a math definition"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4102]
---

## Goal
probes-numeric/t1531-t-tiny-p A84 `CONFIDENCE.T(1E-300,1,100000)` and A108 `CONFIDENCE.T(1E-300,1,100001)` pass as
R3 cells (decision 0188 R3, ruled by decision 0267 item 3 as narrowed by decision 0301), or stay Excel's if the math
does not show Excel more than G = 1e-4 off.

## Context
- T-3386 (decision 0311) marked the other tiny-p T cells but left these two: CONFIDENCE.T has no Gate A definition, so
  `rustsheet-accuracy math` writes no math value for them and an R1(c) mark would not count.
- Excel 0.250207797808864 and 0.2502065076126338; the engine 0.11761635439867599 and 0.11761576226932313, which is
  T.INV.2T's committed math at A82 and A106 (37.19355699853806754, 37.19355571717935502) divided by √n. Excel is
  about 1.13 relative off.
- T-4102 adds CONFIDENCE.T's definition (T.INV.2T(alpha, n-1)·sd/√n).

## Acceptance criteria
- [ ] After T-4102, `cd tools && uv run rustsheet-accuracy math --suite probes-numeric --function CONFIDENCE.T` commits
      math values for A84 and A108.
- [ ] Each cell whose committed math shows Excel more than G off and the engine within ε gets its own line in
      eval/accuracy/math-over-excel.tsv under statistics.md "Tiny-p quantiles: Excel is grossly wrong", with a
      decision record for the protected path; that entry's T-3386 region names it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other CONFIDENCE.T cells (T-3415, T-3421).
