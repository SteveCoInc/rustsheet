---
id: T-4930
title: "A written-empty argument of the discount securities, ACCRINT, ACCRINTM and the T-bill functions follows Excel (probe the ToolPak #N/A rule)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A written-empty argument of DISC, INTRATE, PRICEDISC, RECEIVED, YIELDDISC, PRICEMAT, YIELDMAT, ACCRINT, ACCRINTM,
TBILLEQ, TBILLPRICE and TBILLYIELD gives what Excel gives. A small probe decides, with its predictions committed
before the fill. The engine should stop reading an empty argument as 0 where the ToolPak rule says `#N/A`.

## Context
- The rule under test (dates.md "A written-empty argument of the ToolPak date functions", T-4870; numbers.md "A
  written-empty argument of a ToolPak function is #N/A", T-4873):
  - a required argument written empty, or a LET or LAMBDA variable bound to nothing, is `#N/A` at its own turn in the
    function's read order;
  - over an array it lifts per element;
  - a written-empty optional argument (basis, ACCRINT's calc_method) is omitted.
  XIRR's empty guess already follows the optional half (financial.md "XIRR (and IRR): an empty guess argument is the
  default 0.1 for XIRR", probes/xirr-irr-empty-guess A1, A7).
- Blind spot: no eval cell calls these twelve functions with an empty argument (gap triage 2026-11-07, `make eval` on
  origin/main 1e0e3f023).
- Ours today (scratch eval, unverified):
  - read as 0 and giving a number: `INTRATE(,43831,95,100)` 0.00043858633876652463,
    `PRICEMAT(39448,43831,,0.06,0.061)` -174.56593533487296;
  - giving `#NUM!`: `DISC(39448,43831,,100)`, `PRICEDISC(39448,43831,,100)`, `RECEIVED(39448,43831,1000,,0)`,
    `YIELDDISC(39448,43831,95,,0)`, `YIELDMAT(39448,43831,39000,0.06,,0)`, `ACCRINTM(39000,39448,0.1,,0)`,
    `ACCRINT(39000,39200,39448,0.1,,2)`, `ACCRINT(39000,,39448,0.1,1000,2)`, `ACCRINT(39000,39200,39448,0.1,1000,,0)`,
    `TBILLPRICE(39448,39500,)`, `TBILLYIELD(39448,39500,)`, `TBILLEQ(,39500,0.05)`;
  - empty optional, already the omitted value: `DISC(39448,43831,95,100,)` 0.00416666666666667,
    `ACCRINTM(39000,39448,0.1,1000,)` 122.50000000000001, `ACCRINT(39000,39200,39448,0.1,1000,2,0,)`
    122.50000000000001.
- The read orders are already evidenced, so the predictions follow from them:
  - The seven discount functions read the basis first, then the dates, then the amounts. ACCRINTM reads issue,
    settlement, rate, par, then basis. (financial.md "Discount securities check order", probes/discount-argument-check-order.)
    So `DISC(,43831,95,100,5)` is `#NUM!`, `DISC(-1,,95,100)` is `#NUM!` and `DISC(39448,,1/0,100)` is `#N/A`.
  - ACCRINT reads calc_method first, then the dates as read (financial.md "ACCRINT check order: calc_method first, dates
    as read", T-2676). So `ACCRINT(,39200,39448,0.1,1000,2,0,"1")` is `#VALUE!`.
  - The T-bills read settlement, maturity and discount in order (financial.md "Analysis ToolPak securities functions
    check frequency and dates first"). So `TBILLEQ(-1,,0.05)` is `#NUM!`.
- Code: `discount_args`, `maturity_args`, `accrintm_of`, `tbill_arguments` (functions/financial/discount.rs) and the
  ACCRINT reader (financial/accrint.rs), all on `lift_call`. Reuse `Model::empty_arguments` with
  `written_empty_or`/`written_empty_na` (functions/util/empty_args.rs), and whatever T-4929 adds to the financial
  readers if it lands first. Never copy them (directive 3).
- How it avoids fitting: each probe cell is `#N/A`, an error the evidenced read order gives, or the omitted call's value.
  The fill decides, and no numeric constant is tuned.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) covers the following, with the cells above among them:
      - each required argument of DISC, PRICEMAT, ACCRINT, ACCRINTM and TBILLEQ written empty, one at a time;
      - one empty-amount row for INTRATE, PRICEDISC, RECEIVED, YIELDDISC, YIELDMAT, TBILLPRICE and TBILLYIELD;
      - the empty basis of the nine functions that take one, and ACCRINT's empty calc_method;
      - the order rows above;
      - one lift row, `ROWS(DISC({39448;39500},43831,,100))`;
      - the omitted-variable rows `LET(e,,DISC(39448,43831,e,100))` and
        `LAMBDA(a,b,TBILLPRICE(39448,a,b))(39500,)`.
- [ ] The predictions are committed in a commit before the oracle fill. They are `#N/A` for a required empty argument
      at its turn in the evidenced read order, and the omitted call's value for an empty optional.
- [ ] After the fill every probe cell passes. financial.md gets one entry stating the rule for these twelve functions
      with its cells, or, if Excel reads empties as 0, saying so (then no engine change is made).
- [ ] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- The COUP* functions, PRICE, YIELD, DURATION and MDURATION (T-4929).
- EFFECT, NOMINAL, DOLLARDE, DOLLARFR, CUMIPMT, CUMPRINC, AMORLINC and AMORDEGRC (T-4931).
