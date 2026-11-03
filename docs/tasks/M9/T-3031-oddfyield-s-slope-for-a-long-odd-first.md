---
id: T-3031
title: "ODDFYIELD's slope for a long odd first period at frequency 2 and 4 (T-2007 follow-up)"
milestone: M9
lane: functions-financial
status: done
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
- [x] The listed cells are within half a unit of the 15th significant digit of Excel, or the task is blocked
      time-boxed (decision 0056) with the candidates recorded in financial.md.
- [x] ODDFYIELD lines in eval/baseline only shrink.
- [x] `make check` green; reviewers approve.

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

## Outcome
Attempt 2 (the time-box's second attempt, decision 0056) found it; attempt 1 is above.
ODDFYIELD's slope for a long first period is found (a structural model, decision 0188 R4 kind (c)): T-2007's
regular-coupon slope plus a regular coupon at every quasi-coupon date from the settlement's next one Q (`nq` more
coupons ahead of the odd one), with every exponent of the slope, the redemption's too, shifted by the basis' days from
Q to the first coupon over E (`OddFirstSchedule::slope_shift` from `yield_solver::quasi_shift`, and `nq` extra coupons in `dprice`; financial.md "ODDFYIELD stops short of
ODDFPRICE's root"). A short first period, and a long one with the settlement in its last quasi period, are unchanged.
- How it was found: on the zero-coupon row probes/oddf-date-order-domain-sweep Y!A22:O22 the slope is the
  redemption's alone and the start and price are known, so a Newton run in a scratch lab (engine schedule fields
  dumped, libm powers) gives the shift each cell needs: 1 on bases 0 and 4 and 366/365, 366/360, 92/91, 92/90,
  92/91.25, 183/182.5 on bases 1-3, the quasi period's days over E. The coupon terms were then chosen from 384 slope
  forms (lead amount, lead exponent, coupon shift, extra coupons, redemption shift) on the 43 coupon-bond long-first
  cells of that probe and PRICE_YIELD.xlsx; the shift alone, on any subset of coupon terms, leaves them 1e-12 to 1e-10
  off.
- Scores (engine, eval): ironcalc-calc `FINANCIAL/PRICE_YIELD.xlsx` 339 → 352 of 352 (K8-K11, K29-K33, K49-K52 pass);
  probes/oddf-date-order-domain-sweep 1196 → 1256 of 1260 (Y!A2:O2, A22:O22, A33:O33, A34:O34 pass; left: Y!I29,
  Y!N29 and the ODDFPRICE cells P!A29, P!E29, all 1 ulp); probes/oddfyield-solver-sweep unchanged at 545 of 547 (short
  first periods; its listed cells already passed on main: S!A9, A75-A83, B1, C21, C35, C50, C52, C85, E1, F1, E32, E33,
  E40, F24, F32, F33, F46, F50 all pass; S!C23 and S!E23 are 1 ulp).
- R4 guards. Hold-out (R4.1): no split was fixed before the lab scored, so the hold-out is the fresh sweep below,
  drawn and filled after the model was frozen and never scored before it: train (the 73 long-first cells with `nq` >= 1
  of the two cases above, 58 distinct bonds in the lab) 73 of 73 pass, hold-out 157 of 160 (bit-exact or within the harness' ulp tolerance; margin
  stated before scoring: within 5 points of train). The correctly rounded candidate (ODDFPRICE's root) is not Excel's
  value on these cells: Excel stops short of it by 1e-12 to 1e-10 (financial.md; T-2007's root-landing solver scored
  0 of 99). Fresh sweep (R4.2): probes-numeric/t3031-r4-oddfyield-long-first, 160 seeded calls (2 to 5 quasi periods in
  the first period, `nq` 0 to 4, every frequency and basis, coupon 0 or 0.001-0.2, price 20-200, redemption 50-200),
  excel-graph 2026-11-03: 157 pass; without the extra coupons 72. Gate A (R2): 160 of 160 inside the envelope: 157
  within 15 digits of Excel, S!A80 1 ulp, and S!A61 and S!A70 (month-end schedules, T-3988) within 9e-11 and 1.2e-10
  relative of ODDFPRICE's root (the engine's ODDFPRICE residual over its central-difference slope; `rustsheet-accuracy
  math` has no ODDFYIELD model, so the case has no math sidecar). No `FITTED` tag: no constant was fitted, the shift is
  a day count. Reproducibility floor: no step targets a difference below 4 ulp. No keep-matched branch: the
  conditions are the schedule's own (`nq`, Q), not previously matching cells.
- Not done here, filed: T-3988 (the two month-end cells), T-3989 (a committed fit-lab `oddfyield` kernel; this lab was
  a scratch dump like T-2007's), T-3990 (probe ODDFYIELD's `#NUM!` at extreme prices).
- Eval delta: +73 cells (13 PRICE_YIELD, 60 oddf-date-order-domain-sweep); new case t3031-r4-oddfyield-long-first
  (157 of 160).
