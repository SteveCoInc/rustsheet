---
id: T-4929
title: "A written-empty argument of the COUP* functions, PRICE, YIELD, DURATION and MDURATION follows Excel (probe the ToolPak #N/A rule)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A written-empty argument (`COUPDAYS(,43831,2)`, `PRICE(39448,43831,,0.065,100,2)`, `DURATION(..,2,)`) of COUPDAYBS,
COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD, PRICE, YIELD, DURATION and MDURATION gives what Excel gives. A small
probe decides, with its predictions committed before the fill. The engine should stop reading an empty argument as 0
where the ToolPak rule says `#N/A`.

## Context
- The rule under test (dates.md "A written-empty argument of the ToolPak date functions", T-4870; numbers.md "A
  written-empty argument of a ToolPak function is #N/A", T-4873):
  - a required argument written empty, or a LET or LAMBDA variable bound to nothing, is `#N/A` at its own turn in the
    function's read order: an earlier argument's error wins, a later one loses;
  - over an array the `#N/A` lifts per element;
  - a written-empty optional argument is omitted.
  It holds in every ToolPak family probed so far: date, math and the IM* functions.
- In financial, the only written-empty evidence is XIRR's: `XIRR(v,d,)` is the default guess 0.1, the omitted value. The
  non-ToolPak RATE and IRR read an empty guess as 0 instead (financial.md "XIRR (and IRR): an empty guess argument is the
  default 0.1 for XIRR", probes/xirr-irr-empty-guess A1, A7). This is the "optional is omitted" half of the rule.
- Blind spot: the generator never writes an empty argument, and no eval cell calls these ten functions with one (gap
  triage 2026-11-07, `make eval` on origin/main 1e0e3f023).
- Ours today (scratch eval, unverified):
  - read as 0 and giving a number: `COUPDAYS(,43831,2)` 180, `COUPPCD(,43831,2)` 0,
    `PRICE(39448,43831,,0.065,100,2)` 46.41288364785247, `DURATION(39448,43831,0.08,,2)` 9.183673469387756;
  - giving `#NUM!`: `COUPDAYS(39448,,2)`, `COUPDAYS(39448,43831,)`, `COUPNUM(39448,43831,)`, `COUPDAYS(,,)`,
    `PRICE(39448,43831,0.0575,0.065,,2)`, `YIELD(39448,43831,0.0575,95,100,)` (frequency read as 0);
  - empty basis, already the omitted value: `COUPDAYS(39448,43831,2,)` 180, `COUPNCD(39448,43831,2,)` 39630,
    `PRICE(39448,43831,0.0575,0.065,100,2,)` 93.81687119013684, `DURATION(39448,43831,0.08,0.09,2,)` 7.758246499500057,
    `MDURATION(39448,43831,0.08,0.09,2,)` 7.424159329665127.
- The read orders are already evidenced, so the predictions follow from them without new assumptions:
  - The COUP* functions read settlement, maturity, frequency, basis left to right and check each as they read it
    (financial.md "Analysis ToolPak securities functions check frequency and dates first"). So `COUPDAYS(-1,,2)` and
    `COUPNUM(40000,39000,)` are `#NUM!`, and `COUPDAYS(,1/0,2)` is `#N/A`.
  - PRICE and YIELD read the basis first, and DURATION and MDURATION read it last, after checking the frequency
    (financial.md "PRICE, YIELD, DURATION and MDURATION check order"). So `PRICE(,43831,0.0575,0.065,100,2,9)` and
    `DURATION(39448,43831,0.08,0.09,3,)` are `#NUM!`.
- Code: `coupon_call`/`coupon_args` and `bond_args` (functions/financial/financial_bonds.rs), both on `lift_call`. Reuse
  `Model::empty_arguments` with `written_empty_or`/`written_empty_na` (functions/util/empty_args.rs) inside the
  per-element reader, as `edate_of` does (date_and_time/month_shift.rs). Never copy them (directive 3).
- How it avoids fitting: each probe cell is `#N/A`, an error the evidenced read order gives, or the omitted call's value.
  The fill decides, and no numeric constant is tuned.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) covers the following, with the cells above among them:
      - each required argument of COUPDAYS, COUPNUM, COUPPCD, PRICE, YIELD and DURATION written empty, one at a time,
        plus one all-empty row per function;
      - the empty basis of all ten functions;
      - the order rows above: `COUPDAYS(-1,,2)`, `COUPNUM(40000,39000,)`, `COUPDAYS(,1/0,2)`, `COUPDAYS(1/0,,2)`,
        `PRICE(,43831,0.0575,0.065,100,2,9)`, `DURATION(39448,43831,0.08,0.09,3,)`;
      - one lift row, `ROWS(COUPDAYS({39448;39500},43831,))`;
      - the omitted-variable rows `LET(e,,COUPNUM(39448,43831,e))` and
        `LAMBDA(a,b,PRICE(39448,43831,a,b,100,2))(0.0575,)`.
- [ ] The predictions are committed in a commit before the oracle fill. They are `#N/A` for a required empty argument
      at its turn in the evidenced read order, and the omitted call's value for an empty basis.
- [ ] After the fill every probe cell passes. financial.md gets one entry stating the rule for these ten functions with
      its cells, or, if Excel reads empties as 0, saying so (then no engine change is made).
- [ ] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- The discount securities, ACCRINT, ACCRINTM and the T-bills (T-4930), and the rate conversions, CUMIPMT/CUMPRINC and
  AMORLINC/AMORDEGRC (T-4931).
- ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, XNPV and FVSCHEDULE: file a follow-up if the rule holds here.
- Teaching the generator to write empty arguments.
