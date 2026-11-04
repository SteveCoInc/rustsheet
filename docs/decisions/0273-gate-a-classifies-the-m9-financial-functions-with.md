# 0273 — Gate A classifies the M9 financial functions with no table

- **Status:** accepted (2026-11-04, T-3797). Amends `eval/accuracy/exempt.tsv`, a protected path (decision 0199).
- **Context:**
  - Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs every in-scope numeric catalog function to have a
    reference table or an `exempt.tsv` line. Decisions 0228, 0270, 0271 and 0272 classified the other groups.
  - T-3797 listed 51 M9 financial functions. A recount after decision 0272 (`missing tables 163`) found all 51 still
    with neither.
  - The solver pattern is settled: T-3535 (NPER, RATE) and T-3586 (IRR, XIRR, with array and dated-flow literals). The
    generator's joint `draw` (`accuracy/model.py`) already expresses date-order constraints, so the bond tables need no
    generator task first.
- **Decision:** each of the 51 functions is classified below.
  - **`exact`, 6 functions: COUPNUM, COUPNCD, COUPPCD, COUPDAYBS, COUPDAYS, COUPDAYSNC.** Each returns a count, a whole
    day serial, or a day count. On bases 0, 1 and 4 a day count is whole. On bases 2 and 3 COUPDAYS is 360 or 365 over a
    frequency of 1, 2 or 4 (at most two fractional bits), and COUPDAYSNC is that less a whole count. Every result is
    exact in binary, with no rounding question. This is the DAYS360 case of decision 0228. The schedule rules (the
    calendar's edges, basis 1's chained period) are semantics that the eval suites check (financial.md "Coupon schedules
    at the calendar's edges", "COUPDAYS basis 1 counts from the chained next coupon", "Coupon dates keep the maturity's
    day").
  - **Needs a table, 45 functions,** filed as eight table tasks by family:
    - T-4109, annuity: FV, PV, PMT, IPMT, PPMT, ISPMT, PDURATION;
    - T-4110, annuity and rates: CUMIPMT, CUMPRINC, RRI, EFFECT, NOMINAL, DOLLARDE, DOLLARFR. DOLLARDE and DOLLARFR
      have no semantics entry that rules them exact, so they get a table, which is T-3797's default;
    - T-4111, depreciation: DB, DDB, SLN, SYD, VDB, AMORDEGRC, AMORLINC. The math is the documented algorithm with
      Excel's rounding steps (DB's 3-digit rate, AMORDEGRC's ROUND-like charge);
    - T-4112, discount securities and T-bills: DISC, INTRATE, RECEIVED, PRICEDISC, YIELDDISC, TBILLEQ, TBILLPRICE;
    - T-4113, bonds: PRICE, YIELD, DURATION, MDURATION. YIELD is a solver;
    - T-4114, interest at maturity: ACCRINT, ACCRINTM, PRICEMAT, YIELDMAT, TBILLYIELD;
    - T-4115, odd-period bonds: ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, with a joint draw for the date order.
      ODDFYIELD and ODDLYIELD are solvers;
    - T-4116, cash-flow arrays: NPV, XNPV, MIRR, FVSCHEDULE, on T-3586's one-row array literals. None is `composite`,
      since a literal drives each.
- **Consequences:**
  - `exempt.tsv` gains 6 lines. `missing tables` falls from 163 to 157, with no stale or bad exemption.
  - With decisions 0270 to 0273, every M9 function that Gate A counted as missing is exempt or named by an open table
    task. What remains of Gate A is building those tables, not classifying.
  - As in decision 0228, an `exact` function that later shows a rounding question loses its line to a table task.
    That needs a new decision record, because the path is protected.
