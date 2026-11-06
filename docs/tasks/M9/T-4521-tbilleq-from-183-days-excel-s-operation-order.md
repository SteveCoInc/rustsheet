---
id: T-4521
title: "TBILLEQ from 183 days: Excel's operation order to the bit (accuracy/TBILLEQ/s1 A39, A127)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TBILLEQ from 183 days matches Excel to the bit on accuracy/TBILLEQ/s1 and on the existing TBILLs.xlsx and probe
cells. That means finding the operation order Excel actually uses on the long branch.

## Context
- financial.md "TBILLEQ and TBILLYIELD" documents the long branch as `price = 100 - rate*DSM*100/360`, `t = DSM/Y`,
  `(-t + sqrt(t*t - (2*t - 1)*(1 - 100/price)))/(t - 0.5)`, fitted on ironcalc-calc TBILLs.xlsx. The engine
  (`tbilleq_of`, functions/financial/discount.rs) computes it that way.
- T-4112's table shows the form is not Excel's everywhere (financial.md "Discount securities and T-bills: amount
  domains (T-4112)"):
  - A39 `TBILLEQ(33842,34067,0.213177283226975)` (225 days): Excel 0.24375066975421497, engine 0.24375066975421592
    (about 30 ulp). Excel's value comes out exactly when the price is computed as `100 - rate*100*DSM/360`.
  - A127 `TBILLEQ(45351,45716,0.178249140751279)` (365 days, 2024-02-29 to 2025-02-28): Excel 0.20960730738258349,
    engine 0.20960730738258393. No price variant tried, Y = 366 included, explains it.
  - Over the table's 38 long-branch numeric points, `rate*100*DSM/360` matches 37 and the documented form 36.
- Both points pass Gate A only through the math envelope, so this is fidelity work, not a gate failure.

## Acceptance criteria
- [ ] An operation order for TBILLEQ's long branch that is bit-exact on accuracy/TBILLEQ/s1 (A39 and A127 included),
      TBILLs.xlsx D2:D31 and the excel-graph probes cited in "TBILLEQ and TBILLYIELD"; probe new cells if candidates tie.
- [ ] financial.md "TBILLEQ and TBILLYIELD" states the order with its evidence; the engine follows it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TBILLEQ below 183 days, TBILLPRICE and TBILLYIELD (bit-exact today).
