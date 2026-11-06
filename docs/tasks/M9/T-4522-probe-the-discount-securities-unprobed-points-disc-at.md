---
id: T-4522
title: "Probe the discount securities' unprobed points: DISC at redemption 0, a T-bill discount of 0, a 30/360 day count of 0"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result is known, and recorded in financial.md, for the three discount-security points that T-4112's Gate A
definitions leave without a math value (tools/src/rustsheet_tools/accuracy/defs/discount.py, `NoValueError`).

## Context
- DISC at a redemption of exactly 0 with a positive price and valid dates (`DISC(43466,43831,95,0)`): unprobed. A
  negative redemption is `#NUM!` (gen-m9 31474c20fc1dafd6 A376), and so is a price of 0.
- TBILLEQ, TBILLPRICE (and TBILLYIELD's price) at a discount of exactly 0 with valid dates (`TBILLPRICE(45000,45100,0)`):
  unprobed. A negative discount is `#NUM!` (gen-m9 31474c20fc1dafd6 A241, accuracy/TBILLEQ/s1 A111). The engine
  returns `#NUM!` (`tbill_arguments`, `rate <= 0`).
- A 30/360 day count of 0 on bases 0 and 4 (settlement on the 30th, maturity on the 31st of the same month, e.g.
  `DISC(43130,43131,95,100,0)`, and INTRATE, YIELDDISC, PRICEDISC and RECEIVED likewise): unprobed. The engine returns
  `#DIV/0!` for DISC, INTRATE and YIELDDISC, and divides by nothing for PRICEDISC and RECEIVED.
- Sources: financial.md "Discount securities and T-bills: amount domains (T-4112)", "Discount securities check order".
- T-4114 left these without a math value too (defs/discount.py, defs/accrint.py): TBILLYIELD at a price of 0 or below
  (`TBILLYIELD(45000,45100,0)`), YIELDMAT at a price of 0, ACCRINTM and ACCRINT at a par of 0, ACCRINTM with equal
  dates and a rate or par at most 0, ACCRINT at a frequency other than 1, 2 or 4 or a basis from 5, and PRICEMAT and
  YIELDMAT where the 30/360 count DSM = DIM - A is 0 or below (the engine's `#DIV/0!` for YIELDMAT). Source:
  financial.md "ACCRINT, ACCRINTM, PRICEMAT and YIELDMAT: amount domains (T-4114)".

## Acceptance criteria
- [ ] An excel-graph probe case covering the three points on every function they apply to (bases 0 and 4 for the
      day count) is filled and committed, using the `excel-probe` skill.
- [ ] financial.md records each result; a mismatch with the engine gets a functions-financial task; the definitions
      in defs/discount.py encode the evidenced results instead of `NoValueError`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other discount-security domains (already evidenced).
