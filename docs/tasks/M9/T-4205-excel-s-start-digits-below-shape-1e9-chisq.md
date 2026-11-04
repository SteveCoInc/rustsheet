---
id: T-4205
title: "Excel's start digits below shape 1E9: CHISQ.INV.RT(1E-300,df) for df 5E6 to 1E9 within 15 digits"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`CHISQ.INV.RT(1E-300,df)` for df 5E6 to 1E9 (probes/t3350-chisq-inv-huge-df A205, A217, A229, A241, A253, A265, A277, A289) is Excel's value within 15 significant digits.

## Context
Since T-3409 these cells take the right regime: Excel returns its Wilson-Hilferty start x0 where the density at x0
underflows (`huge_shape.rs` `huge_shape_quantile`, now from shape 1E5), 2.6E-4 above the true quantile. But the start's
higher terms (`WILSON_HILFERTY_SPREAD`, `WILSON_HILFERTY_INVERSE_SHAPE`, `WILSON_HILFERTY_SQUARE`, `WILSON_HILFERTY_CUBE`)
were fitted (T-3070) on closed-form cells from shape 1.5E10, where their 1/a terms are tiny; at shape 2.5E6 to 5E8 ours
is 1E-9 to 3.4E-9 relative below Excel (A205 ours 5119417.184081892, Excel 5119417.201267746). Find Excel's start
formula (its 1/a and z terms) so one form fits both ranges, or decide by decision record that these 8 cells stay
off. Related: T-3272 (the closed form's digits from 1E9). docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV
at a huge shape". A fit owes decision 0188 R4's hold-out and fresh sweep.
The fresh sweep of T-3409 adds the same kind of cell (the start, right type, digits off): probes-numeric/
t3409-r4-gamma-inv-search-sweep Calc!A231, Calc!A262:A263, Calc!A266, Calc!A268, Calc!A271, Calc!A276:A279
(`CHISQ.INV.RT` at p from 1E-295 to 1E-251, df 3.7E7 to 9.7E8), and probes-numeric/t3409-r4-gamma-inv-search-sweep
Calc!A226 `CHISQ.INV.RT(6.19504333347244E-280,9522000000)` at shape 4.8E9 (the closed form from 1E9, T-3272's range).

## Acceptance criteria
- [ ] A205, A217, A229, A241, A253, A265, A277, A289 of probes/t3350-chisq-inv-huge-df and the sweep cells above are within 15 digits of Excel, and no closed-form cell from 1E9 that matches now regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
