---
id: T-3031
title: "ODDFYIELD's slope for a long odd first period at frequency 2 and 4 (T-2007 follow-up)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
ODDFYIELD's visibly-off cells left after T-2007 come within 15 digits of Excel: the long odd first periods at
frequency 2 (and 4), and the sweep cells near a zero yield.

## Context
T-2007 (2026-10-28) fitted ODDFYIELD as YIELD's Newton (`excel_yield_newton`, from `yield_start` over
`yield_start_years`, a step under 1e-10 not taken) with a slope that weighs the odd first coupon as a regular one
(`OddFirstSchedule::dprice`; financial.md "ODDFYIELD stops short of ODDFPRICE's root"). That fits the short first
periods and the annual long ones, but not:
- ironcalc-calc `FINANCIAL/PRICE_YIELD.xlsx` ODDFPRICE_ODDFYIELD!K8-K11, K29-K33, K49-K52: frequency 2, issue
  2020-01-15, first coupon 2021-03-01 (a long first period over three quasi-coupon periods), settlement on a
  quasi-coupon date. Ours is off by 1e-11 to 1e-10 on both sides. Tried and rejected in T-2007: YIELD's own slope over
  the regular `CouponSchedule::new(settlement, maturity)` (worse: 321 cells pass in that file against 325), and the
  odd schedule's slope with a regular coupon at every quasi-coupon date from settlement (325, no better). The committed
  slope (the lead weighed as a regular coupon at its own exponent `nq + y`) has no evidence for nq > 0.
- ODDFYIELD's #NUM! now comes from YIELD's Newton leaving ODDFPRICE's domain (no bisection fallback since T-2007);
  no ODDFYIELD oracle cell exercises it. Probe extreme prices (1, 5, 500, 5000, a zero coupon at a tiny price) on
  excel-graph and record the rule in financial.md.
- T-2007's lab was a temporary dump, not a committed `oddfyield` kernel in fitlab/kernels/solvers.py; commit one
  here so the scores can be reproduced.
- probes/oddfyield-solver-sweep S!A9, A75, A76, A77, A78, A79, A80, A81, A82, A83 (the 5% semiannual bond at prices
  near 124-132, its yield near 0; listed one by one because a hyphen span names its ends only, decision 0136),
  B1, C21, C35, C50, C52, C85, E1 and F1 (the inverted ODDFPRICE at a yield of 0), E32, E33, E40, F24, F32, F33, F46,
  F50 (NumericClose or NumericMismatch on the 2026-10-28 run).
- probes/oddf-date-order-domain-sweep (T-3652, excel-graph 2026-11-03) Y!A2:O2, A22:O22, A33:O33, A34:O34: a long
  first period with the settlement before the quasi-coupon date, at frequency 1, 2 and 4 and every basis (prices 98,
  1E-6 and 150, and a 0% coupon at 98). Ours is off by about 1e-9 relative on both sides; annual is off too, so the
  committed slope is wrong for any long first period whose settlement precedes the quasi-coupon date, not only at
  frequency 2 and 4.
- The rest of the sweep is within a few ulps; YIELD's slope operation order (`t * 100 * rate / m`) is worse (319
  sweep cells pass against 320).
Lab: the T-2007 method (dump `OddFirstSchedule`'s fields from `fn_oddfyield`, re-run Newton in Python with
`caret_pow`) scores a candidate in seconds; the long-period quasi-coupon count is `offset` (nq).

## Acceptance criteria
- [ ] The listed cells are within half a unit of the 15th significant digit of Excel, or the task is blocked
      time-boxed (decision 0056) with the candidates recorded in financial.md.
- [ ] ODDFYIELD lines in eval/baseline only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD (T-1592) and ODDFPRICE's own digits (T-1374).

## Attempt 1 (2026-11-03, decision 0056)
No bit-exact gain; the time-box's second attempt is still open. Method: an env-gated lab hook in
`OddFirstSchedule::dprice` (not committed) scored in the engine with `rustsheet-eval show` on PRICE_YIELD.xlsx,
probes/oddf-date-order-domain-sweep and probes/oddfyield-solver-sweep, plus a schedule dump (`OddFirstSchedule`
fields and the start) re-run in Python/mpmath.
- Candidates (long first periods only, `offset > 0`), cells passing per case (committed: 339 / 1196 / 545): the
  exact lead `lead` at `lead_exponent`; a regular coupon at `y`; no lead; `(offset + 1)` coupons at the lead's
  exponent; `lead` at `y`; a regular coupon at every quasi-coupon date from settlement; `/ m` before the power;
  a regular coupon at `y + 1`; one at `lead_exponent - 1`; `y * lead` at the lead's exponent; `lead - offset ×
  coupon`. Every one: 339 / 1196 / 545, 0 long-first cells gained (max relative error moves between 1.3e-9 and
  3.5e-9 on PRICE_YIELD.xlsx).
- Starts: YIELD's start over issue-maturity, first coupon-maturity, settlement-first coupon, issue-first coupon, and
  the coupon rate: 0 of the 73 long-first cells (13 PRICE_YIELD.xlsx, 60 sweep Y rows 2, 22, 33, 34).
- Finding (financial.md "ODDFYIELD stops short of ODDFPRICE's root", long first periods, zero coupon): on Y!A22:O22
  (rate 0, so the committed slope is exact) Excel still differs, and ends above the root in G22, I22, K22, L22,
  where an exact step from YIELD's start (above the root) ends below it, so the long-first difference is in the Newton itself (its slope's other terms or its start), not in the
  lead coupon's weight. Next: a second attempt should fit the zero-coupon long-first cells alone first (one term,
  no lead), e.g. the redemption term's exponent in the slope (`front_exponent` from a different `nq`/`y`, or
  `t` vs `t + 1` in the power), then carry the result to the coupon rows.
- Not done: the fitlab `oddfyield` kernel and the extreme-price #NUM! probe (still acceptance work).
