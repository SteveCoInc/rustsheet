---
id: T-4115
title: "Gate A tables, odd-period bonds: ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0273 (T-3797) classified these functions as needing a table.
- Date arguments are integer serials. An order constraint (settlement < maturity, issue < settlement < first coupon) is a
  Definition's joint `draw` or a `domain` rule (`accuracy/model.py`), as T-3535's NPER and RATE use (`defs/annuity.py`).
  Basis is an `int` parameter over 0..4. Reuse `defs/yearfrac.py` for the day counts (directive 3).
- Solvers follow the IRR and RATE pattern (T-3535, T-3586): `solver=True`, 300 points, and Excel's root choice cited from
  a semantics entry. A point with several roots is not drawn without one.
- Where Excel's own arithmetic (a rounding step, an operation order) puts it far from the math, list the points in the
  Outcome as R3 candidates for an owner ruling and do not mark them (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- The dates need issue < settlement < first coupon, with the first coupon on the maturity's schedule: a joint `draw`.
  ODDFYIELD and ODDLYIELD are solvers. Sources (financial.md): "ODDFPRICE and ODDFYIELD need issue < settlement <
  first_coupon", "ODDFPRICE and ODDFYIELD need the first coupon on the maturity's schedule", "ODDLPRICE and ODDLYIELD: a
  quasi-coupon date past the calendar is #NUM!", "ODDFPRICE and ODDLPRICE at a huge yield", "ODDFYIELD and ODDLYIELD at a
  price far from par", "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order", "ODDFYIELD stops short of
  ODDFPRICE's root", "ODDFYIELD's #NUM! at extreme prices".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve. (`make check-fast` green locally; reviewers pending)

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/odd_bonds.py`: one definition each for ODDFPRICE, ODDFYIELD, ODDLPRICE and
  ODDLYIELD. The formulas are Microsoft's odd-first and odd-last-period ones on the schedules and day counts of the
  engine's oddbonds.fs port (`OddFirstSchedule`, `coup_number`, `odd_l_func`), which match Excel bit-exact, taken as
  exact rationals: the short and long first period (quasi-coupon walk, DCNL and ANL, DSC on bases 2-3 against 0, 1 and
  4, `coupNumber`'s month-end quasi count, the DFC = E `#NUM!`) and the drifting odd-last walk with basis 0's
  both-dates 30/360 period lengths (new `yearfrac.us_30_360_both`). It reuses bonds.py (`coupons`, `basis_days`,
  `date_serial`, the double-range tests, now public), the fit lab's `schedule`, `add_months` and `month_end` (now
  public) and `annuity.unique_rate`. bonds.py now holds what both modules draw and register with (`value_edge`,
  `price_or_par`, `bond_definition`); PRICE's, YIELD's, DURATION's and MDURATION's tables regenerate byte-identical. ODDFYIELD is ODDFPRICE's unique root (`solver=True`, 300 points; every flow is
  positive and after the dirty price); ODDLYIELD is ODDLPRICE's closed-form inverse.
  - Domain errors, each from a semantics entry: dates off the calendar, frequency, basis, a negative rate or yld, a pr
    or redemption of 0 or below, the date orders, a first coupon off the maturity's schedule, DFC = E, an ODDL*
    quasi-coupon date past 9999-12-31 on bases 0 and 1, ODDFPRICE's redemption power past the double range. No value
    (unprobed): a date before 1901-03-01, a walk past the calendar for ODDF* or on ODDL* bases 2-4, 100·rate, a dirty sum
    or a price past the double range, ODDLYIELD with DSCNL = 0.
  - Pre-check before generating, over the 2,841 literal one-call cells of these functions in eval/suites (probes,
    probes-numeric, gen-m9; 1,755 more skipped as not literal): no math domain error where Excel has a number, and no
    Excel number where the math has a domain error. ODDFPRICE (544 numeric, 422 `#NUM!`), ODDLPRICE (122, 24; 2 Excel
    `#NUM!` cells with a last interest of serial 1 get no value) and ODDLYIELD (112, 40) are within 1E-9 of Excel on
    every numeric cell. ODDFYIELD: 943 within 1E-9, 136 within 1E-4, 21 beyond, 459 errors on both sides, 13 Excel
    `#NUM!` with a root (the documented far-from-par and extreme-price Newton failures), 3 dropped (a root of exactly 0).
  - Tests in tools/tests/test_accuracy_odd_bonds.py (Microsoft's examples, the semantics entries' Excel values, T-3999's
    two 50-digit roots, price/yield inversion on every basis, the evidenced errors and no-value regions, draws).
- Tables `eval/accuracy/tables/{ODDFPRICE,ODDLPRICE,ODDLYIELD}/s1` (200 points each) and `ODDFYIELD/s1` (300), filled
  from excel-graph with 0 failures.
- `rustsheet-eval accuracy`: 898 of the 900 points inside the envelope, 0 failing, 0 table defects; `missing tables` 50
  to 46. The engine is bit-exact with Excel on 897 of 900 (ODDFPRICE 199, ODDFYIELD 298, ODDLPRICE 200, ODDLYIELD 200;
  ODDFPRICE A162 is 1 ulp off, inside). Two ODDFYIELD points are known failures under T-4551, both roots near 0 where
  Excel and the engine stop short of the math: A272 (engine 1.1E-11 relative off Excel) and A289 (2E-13).
- Every Excel error on the tables matches a coded domain, except 15 ODDFYIELD points where Excel's Newton is `#NUM!` at
  a price and redemption far from par (A188, A197, A218, A221, A229, A235, A237, A243, A251, A277, A280, A285, A292,
  A297, A299); the engine matches all 15. New entry in financial.md: "ODDFYIELD is #NUM! at a price and redemption far
  from par".
- New quirk, recorded in financial.md "ODDFYIELD off ODDFPRICE's root when ODDFPRICE counts a quasi period too many":
  with a month-end first coupon, ODDFYIELD's Newton (true-period slope, `coupNumber` price) lands far from ODDFPRICE's
  root (A126; probes-numeric/t3988-r4-oddfyield-fresh-2 S!A46, where ODDFPRICE at Excel's yield is 67.27, not
  102.489). The engine matches both.
- **Excel kept as the target (decision 0300):** 32 points are more than 1e-4 relative off the math, all ODDFYIELD. On the
  table: A126 (5.4, the quirk above) and ten roots near 0 where Excel's absolute step stops short (A129, A132, A143,
  A150, A153, A170, A174, A202, A291, A295; absolute gaps under 1E-10). On the sidecars: probes/odd-yield-far-from-par
  F1-F16 (redemption 1E300, Excel's 100th iterates 236 to 303 against roots near 1E15) and G4 (price 3E4: -3.50 against
  -0.489), probes/oddfyield-extreme-price K5, K6 and L6 (zero-coupon long first periods at a price of 1E-3 or 1E-6,
  0.5% to 9% off), and probes-numeric/t3988-r4-oddfyield-fresh-2 S!A46 (5.05). The engine is bit-exact with Excel on
  every one. No R3 marks; listed in financial.md under both new entries.
- T-3999 (an ODDFYIELD math model and table) is covered by this task: its two roots are unit tests here and the
  t3031 case now has a math sidecar.
- Math sidecars: `rustsheet-accuracy math --function` over the four functions, for probes and probes-numeric: 18 new
  sidecars (2,797 cells, 830 of them math domain errors) and 69 cells added to 10 existing ones, with no existing cell
  changed or removed. `make eval-math-check` prints "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
