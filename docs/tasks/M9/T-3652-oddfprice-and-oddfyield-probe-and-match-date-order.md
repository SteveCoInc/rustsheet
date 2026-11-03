---
id: T-3652
title: "ODDFPRICE and ODDFYIELD: probe and match date-order and domain validation across rate, yld, frequency and basis, against ODDL*"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
ODDFPRICE and ODDFYIELD give `#NUM!` where Excel does for every date ordering and argument domain, and Excel's visible value elsewhere across broad grids of rate, yield or price, frequency and basis, with the differences from the ODDL* pair (which mostly match) explained.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports ODDFPRICE and ODDFYIELD returning values where Excel gives `#NUM!`, and visibly
different numbers across broad combinations of rate, yield, frequency and basis; the ODDL* pair mostly matches.
Done: T-2483 (argument check order of all four; ODDLPRICE before the last coupon), T-1447, T-1457, T-1490 (dates
near 9999), T-2007 and T-3152 (ODDFYIELD's solver), T-2207 (^ kernel), T-2659 and T-3122 (overflow). Open: T-3031
(ODDFYIELD slope for a long odd first period at frequency 2 and 4), T-3043 (last ulps), T-3625 (100th-check stop).
Not swept: every ordering of issue, settlement, first_coupon and maturity (equal dates, settlement before issue,
settlement on the first coupon, first_coupon past maturity, maturity off the coupon schedule), short versus long
odd first periods at each frequency, every basis, a negative or zero rate, yield or price, and a redemption of 0.
Compare each validation rule with ODDLPRICE/ODDLYIELD's, which already match, and with financial.md. Code:
functions/financial/financial_bonds.rs (`fn_oddfprice`, `fn_oddfyield`, `odd_first_yield`).

## Acceptance criteria
- [x] A probe case (excel-graph) covers the date orderings and domain edges above at every frequency and basis.
- [x] Every probed cell has Excel's type, and every number is within half a unit of Excel's 15th digit, except cells handed to T-3031 or T-3043 with a reason.
- [x] financial.md states ODDF* validation next to ODDL*'s.
- [x] Every quirk found is written up in docs/excel-semantics/financial.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Last-ulp fits (T-3043).
- ODDLPRICE/ODDLYIELD changes unless a probe shows a shared bug.

## Outcome
- New probe probes/oddf-date-order-domain-sweep (1260 cells, excel-graph 2026-11-03): ODDFPRICE (sheet P) and
  ODDFYIELD (sheet Y) over 26 date-order and domain scenarios plus yld/pr edges, at frequency 1, 2, 4 by basis 0-4,
  with a short and a long odd first period.
- Finding: Excel wants issue < settlement < first_coupon. A settlement on or before the issue, or on or after the
  first coupon, is `#NUM!`; we priced it (162 cells, P!A5:O11 and Y!A5:O11). Fixed in `odd_first_of`
  (financial_bonds.rs) with a unit test. Every other date ordering and every domain edge (rate, yld, pr, redemption
  of 0 or below) already matched, and ODDF* validation is ODDL*'s with the odd period at the front.
- financial.md: new entry "ODDFPRICE and ODDFYIELD need issue < settlement < first_coupon"; the check-order entry's
  step 4 names the rule.
- Handed off with reasons: Y!A2:O2, A22:O22, A33:O33, A34:O34 (ODDFYIELD's solver on a long first period with the
  settlement before the quasi-coupon date, about 1e-9 relative, annual too) to T-3031; Y!I29, N29 (ulps) to T-3043;
  P!A29, E29 (ODDFPRICE ulps) to T-1374.
- Eval delta: probes +1 case (1260 cells, 1196 pass, 64 new-coverage known failures, all owned); no regressions,
  no other suite changed.
