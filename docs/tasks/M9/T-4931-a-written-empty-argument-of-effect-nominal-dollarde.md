---
id: T-4931
title: "A written-empty argument of EFFECT, NOMINAL, DOLLARDE, DOLLARFR, CUMIPMT, CUMPRINC, AMORLINC and AMORDEGRC follows Excel (probe the ToolPak #N/A rule)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A written-empty argument of the Analysis ToolPak functions EFFECT, NOMINAL, DOLLARDE, DOLLARFR, CUMIPMT, CUMPRINC,
AMORLINC and AMORDEGRC gives what Excel gives. A small probe decides, with its predictions committed before the fill.
The engine should stop reading an empty argument as 0 where the ToolPak rule says `#N/A`.

## Context
- The rule under test (dates.md "A written-empty argument of the ToolPak date functions", T-4870; numbers.md "A
  written-empty argument of a ToolPak function is #N/A", T-4873):
  - a required argument written empty, or a LET or LAMBDA variable bound to nothing, is `#N/A` at its own turn in the
    read order;
  - over an array it lifts per element;
  - a written-empty optional argument (AMORLINC's and AMORDEGRC's basis) is omitted.
  XIRR's empty guess follows the optional half (financial.md "XIRR (and IRR): an empty guess argument is the default
  0.1 for XIRR", probes/xirr-irr-empty-guess A1, A7). The non-ToolPak RATE and NPV read an empty argument as 0
  (financial.md "RATE: an empty guess is guess 0", "NPV's error order"). These eight are the ToolPak ones, so the probe
  needs non-ToolPak controls.
- Blind spot: no eval cell calls these eight functions with an empty argument (gap triage 2026-11-07, `make eval` on
  origin/main 1e0e3f023).
- Ours today (scratch eval, unverified):
  - read as 0 and giving a number: `DOLLARFR(,16)` and `DOLLARDE(,16)` are 0, and `CUMIPMT(0.01,36,8000,1,12,)`
    -833.2493088275287 (type read as 0; type is required in CUMIPMT and CUMPRINC);
  - other errors: `DOLLARDE(1.02,)` `#DIV/0!`; `EFFECT(0.1,)`, `EFFECT(,4)`, `NOMINAL(,4)`, `NOMINAL(0.1,)`,
    `CUMIPMT(,36,8000,1,12,0)`, `CUMIPMT(0.01,36,8000,1,,0)`, `CUMPRINC(0.01,36,8000,,12,0)` and
    `AMORLINC(,39679,39813,300,1,0.15)` `#NUM!`;
  - empty basis, already the omitted value: `AMORLINC(2400,39679,39813,300,1,0.15,)` 360,
    `AMORDEGRC(2400,39679,39813,300,1,0.15,)` 776.
- Read orders: AMORLINC and AMORDEGRC read cost through rate in order, check the values, and only then read the basis
  (financial.md "AMORLINC and AMORDEGRC check order"). So `AMORLINC(2400,-1,,300,1,0.15)` is `#NUM!` and
  `AMORLINC(NA(),39679,,300,1,0.15)` is `#N/A`.
  EFFECT, NOMINAL, DOLLARDE and DOLLARFR read their two numbers left to right, the first one's error winning
  (`Model::two_numbers`, functions/lift/mod.rs). Their order against an empty argument is predicted from that reader.
  CUMIPMT's and CUMPRINC's read order is not evidenced; predict it left to right. If a cell misses, run a second round
  with fresh rows whose predictions are committed first, as T-4870 did.
- Code:
  - `fn_effect`, `fn_nominal`, `fn_dollarde`, `fn_dollarfr` (functions/financial/rate_conversions.rs) through
    `Model::two_numbers` (functions/lift/mod.rs);
  - `fn_cumipmt`/`fn_cumprinc` (financial/cumulative.rs);
  - `fn_amorlinc`/`fn_amordegrc` (financial/amortization.rs).
  Reuse `Model::empty_arguments` with `written_empty_or`/`written_empty_na` (functions/util/empty_args.rs), and whatever
  T-4929 or T-4930 adds if one lands first. Never copy them (directive 3).
- How it avoids fitting: each probe cell is `#N/A`, another error kind, or the omitted call's value. The fill decides,
  and no numeric constant is tuned.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) covers the following, with the cells above among them:
      - each argument of EFFECT, NOMINAL, DOLLARDE and DOLLARFR written empty, one at a time and both;
      - each of CUMIPMT's six arguments written empty, one at a time, and CUMPRINC's start_period and type;
      - each of AMORLINC's first six arguments written empty, and both functions' empty basis;
      - order rows: `DOLLARDE(1/0,)`, `DOLLARDE(,1/0)`, `CUMIPMT(1/0,,8000,1,12,0)`, `AMORLINC(2400,-1,,300,1,0.15)`;
      - one lift row, `ROWS(EFFECT({0.1;0.2},))`;
      - `LET(e,,EFFECT(0.1,e))`;
      - non-ToolPak controls: `PMT(0.01,36,)` and `IPMT(0.01,1,36,)`.
- [ ] The predictions are committed in a commit before the oracle fill. They are `#N/A` for a required empty argument
      at its turn in the read order, the omitted call's value for an empty basis, and today's answers for the controls.
- [ ] After the fill every probe cell passes. financial.md gets one entry stating the rule for these eight functions
      with its cells, or, if Excel reads empties as 0, saying so (then no engine change is made).
- [ ] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- The COUP*/bond functions (T-4929); the discount securities, ACCRINT, ACCRINTM and T-bills (T-4930).
- ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, XNPV and FVSCHEDULE: file a follow-up if the rule holds here.
