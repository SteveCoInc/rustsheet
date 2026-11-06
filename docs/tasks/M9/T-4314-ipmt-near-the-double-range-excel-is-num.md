---
id: T-4314
title: "IPMT near the double range and at tiny rates: Excel is #NUM! where the engine has a value (319 cells of t3523-ipmt-selector-sweep)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IPMT returns `#NUM!` where Excel does near the double range and at subnormal and tiny rates, so the visibly-off cells
below pass.

## Context
T-3523's fresh sweep (probes-numeric/t3523-ipmt-selector-sweep, excel-graph 2026-11-05) found Excel `#NUM!` where the
engine returns a finite number (with T-2587's selector and without it). financial.md "IPMT near the double range and
at tiny rates" records them:
- probes-numeric/t3523-ipmt-selector-sweep.case.json, column B (200, subnormal rates written as kept-subnormal
  differences such as `(7.245817E-308-6.217028E-308)`): Calc!B1:B200
- column C (6, pv and fv 1E+290 to 9.9E+307 at ordinary rates): Calc!C44, C46, C55, C114, C179, C189; e.g. C44
  `IPMT(0.00282609064045807,30,31,2.17430063151056E+307,0,0)`, engine -4.128645778278784E+303. The other 194 column
  C cells have Excel's finite values (one, C139, 13 digits off: see below).
- column D (113, rates 1E-300 to 1E-12): Calc!D1, D3, D6:D7, D10:D11, D13:D14, D18, D20:D21, D23, D25, D28, D31,
  D34:D36, D39:D40, D44, D49, D51, D53:D54, D57:D60, D63:D65, D67, D70:D73, D76, D78:D80, D82:D89, D92, D94:D97, D99,
  D102, D104:D105, D107:D108, D110:D112, D114, D117:D120, D122:D127, D133:D135, D137:D138, D140:D143, D147, D153,
  D155:D156, D158:D161, D163:D170, D174:D180, D183, D185:D186, D191, D194, D196, D198
- C139 `IPMT(0.838240020339469,5,15,1.31635124625391E+294,4.11149200495181E+298,1)`: Excel 2.052481555601429E+295,
  engine 2.052481555601641E+295 (both about 5E-14 from the math, on opposite sides).
Every sweep cell where the engine's T-2243 discounted split declines (234) is Excel `#NUM!`, so the split's own
decline conditions are the first candidate (`discounted_split`, `payment_factor`, `discounted_balance` in
annuity.rs), then an intermediate of Excel's form overflowing or underflowing (`(pv+fv)`, `pv*(1+r)^k`,
`1-(1+r)^-n` at a tiny rate).

## Acceptance criteria
- [x] The condition under which Excel's IPMT is `#NUM!` is identified from these cells and the passing ones of the
      same columns (a short probe where they do not decide it), and IPMT follows it.
- [x] The cells above pass, or the ones that do not are named with their owner; no other cell regresses
      (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PPMT and PMT on the same inputs unless the same condition decides them (file a task if so).

## Outcome
Excel's IPMT is `#NUM!` where its discounted split (T-2243) has no balance or no factor, and IPMT now follows that:
- The rule (financial.md "IPMT is #NUM! where its discounted balance fails"), outside period 1 of type 1 at a
  nonzero rate:
  - `A = (pv+fv)/(1-Q(-nper*L))` is past the double range;
  - `1-Q` is below the normal range;
  - or the rate is subnormal, so `d` is not normal.
- How it was found: the sweep's intermediates in doubles split columns C and D exactly on A's overflow. The largest
  Excel-finite |A| is 1.43E+308; every `#NUM!` cell has A infinite. Column B is A infinite (188), `1-Q` subnormal (11)
  or only the subnormal rate (B127).
- probes/t4314-ipmt-balance-overflow (35 cells, excel-graph 2026-11-06) checks each condition on its own:
  - `IPMT(0.5,1,2,9.9E+307)` is -4.95E+307, `IPMT(0.5,1,2,2*5E+307)` `#NUM!`;
  - pv+fv = 0 keeps a value;
  - a subnormal rate with A = 9.7E+288 is `#NUM!`;
  - `IPMT(1E-300,1,1E-10,1E-300)` (A = 1E+10, `1-Q` subnormal) is `#NUM!`;
  - period 1 of type 1 stays a true 0.
- It also showed that a subnormal interest at the smallest normal rates is a true 0 (`(x)=0` TRUE), so
  `discounted_split` now flushes its interest.
- Code: `discounted_balance` declines on a non-finite A; `period_payments` errs IPMT where `discounted_balance` or
  `payment_factor` declines; `discounted_split`'s interest goes through `flush_subnormal`. Unit test
  `interest_errs_where_the_discounted_balance_fails`.
- Eval delta: cell by cell, report.json of an origin/main c97e0b9c build against this branch, each case's `passed`
  count and its failing cells:
  - probes-numeric/t3523-ipmt-selector-sweep goes from 480 to 799 of 800: all 319 `#NUM!` cells pass. C139 (13th
    digit, Tier B) is left to T-4491.
  - The new probes/t4314-ipmt-balance-overflow passes 28 of 35. The 7 off cells are PMT, PPMT and CUMIPMT on the
    same inputs, where Excel is `#NUM!` too: Sheet1!B2, B4:B6, C7:C8, D3, owned by T-4488.
  - No other cell of any suite changes. `make eval-baseline` records the new case (probes 5593 -> 5600 known
    failures, all 7 new coverage; visible-parity line added).
- Not a fit: the rule is three decline conditions the split already has, read off the evidence; no constant was
  tuned.
