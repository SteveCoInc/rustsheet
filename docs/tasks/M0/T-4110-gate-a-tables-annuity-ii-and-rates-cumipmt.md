---
id: T-4110
title: "Gate A tables, annuity II and rates: CUMIPMT, CUMPRINC, RRI, EFFECT, NOMINAL, DOLLARDE and DOLLARFR"
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
CUMIPMT, CUMPRINC, RRI, EFFECT, NOMINAL, DOLLARDE and DOLLARFR each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/annuity.py` for CUMIPMT and CUMPRINC. Sources (financial.md): "CUMIPMT and CUMPRINC over many periods",
  "CUMIPMT and CUMPRINC work cap", "EFFECT and NOMINAL raise to their power with `^`", "T-1625 probes: VDB, AMORDEGRC,
  FVSCHEDULE, MDURATION, RRI". T-3731 (done) settled CUMIPMT and CUMPRINC at a tiny rate: ulps off, not an R3 case.
- DOLLARDE and DOLLARFR truncate `fraction` and do decimal arithmetic; no semantics entry rules them exact, so they get
  a table (T-3797's default; arrays.md lists their argument lifting).

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green
- [ ] reviewers approve.

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/annuity_cumulative.py`: CUMIPMT and CUMPRINC, the exact sums over the whole
  periods ceil(start)..floor(end) in closed form (constant time in the period count), with the payment from
  `present_value` and the draws from `_annuity` (defs/annuity.py, T-3535). The interest runs with extra digits where
  `m - Σ v^j` cancels. Type 1's period 1 is the whole payment.
- `defs/interest_rates.py`: EFFECT, NOMINAL (npery truncated, `Param.truncates` citing "EFFECT and NOMINAL raise to
  their power with `^`") and RRI. `defs/dollar_fractions.py`: DOLLARDE and DOLLARFR with D the power of ten at or
  above the fraction and the dollar split toward 0.
  - Domain errors (`#NUM!`), each from an eval cell or a semantics entry: CUMIPMT/CUMPRINC at rate 0, a start below 1,
    an end past nper, a start after the end; EFFECT/NOMINAL at rate 0 or an npery that floors to 0, EFFECT's power past
    the double range; RRI at nper <= 0, pv 0 with fv not 0, a negative fv/pv, a growth past the double range.
  - No math value where nothing evidences Excel's result: negative rates (CUMIPMT/CUMPRINC, EFFECT, NOMINAL), a
    negative npery, pv <= 0 or type other than 0/1 for CUMIPMT/CUMPRINC, ceil(start) > floor(end) with start <= end,
    RRI with pv = fv = 0, DOLLARDE/DOLLARFR fractions below 1 (and none drawn past 1E9), powers within a hair of the
    double range's top.
  - Before generating, the definitions ran over the 4,316 literal one-call cells of these functions in eval/suites:
    no math domain error where Excel has a number. One cell
    has a math number where Excel has an error, merged from main after the check:
    probes/t4314-ipmt-balance-overflow B6 `CUMIPMT(0.5,2,2*5E+307,1,1,0)` (math -5E+307, Excel `#NUM!`), where an
    intermediate overflows as in that case's PMT, IPMT and PPMT cells (documented in financial.md "IPMT is #NUM! where
    its discounted balance fails"); its sidecar keeps the math value, as theirs do, and the CUMIPMT definition codes
    no domain for it.
  - Tests in tools/tests/test_accuracy_annuity_rates.py (Microsoft's examples, the semantics entries' Excel values,
    the evidenced domains, the no-value regions, draws).
- Tables `eval/accuracy/tables/{CUMIPMT,CUMPRINC,RRI}/s1` (200 points each) and `{EFFECT,NOMINAL,DOLLARDE,DOLLARFR}/s1`
  (120 each), 1,080 points, filled from excel-graph with 0 failures. Every Excel error (61, all `#NUM!`) is a coded
  domain error; 0 table defects.
- `rustsheet-eval accuracy`: 1,076 of 1,080 points inside the envelope, 0 failing; `missing tables` lower by 7 (121 to 114 on the claimed main, 114 to
  107 after merging origin/main). The engine is bit-exact with Excel on 910 of 1,080 (CUMIPMT 144/200, CUMPRINC 86/200, all others every point).
- Known failures added (4, owner T-4390, which owns the visibly-off tiny-rate CUMIPMT cells of the same cancellation):
  CUMIPMT/s1 A87, A90, A92, A189, at rates 1.5E-12 to 4.9E-10, where `PMT·m - CUMPRINC` cancels and our principal's
  last bits put the interest 1E-9 to 2.5E-6 relative off Excel.
- New semantics entries in financial.md: "CUMIPMT and CUMPRINC argument domains (T-4110)", "EFFECT, NOMINAL and RRI
  near a rate of 0 are far from the math (T-4110)" and "DOLLARDE and DOLLARFR scale the fraction by the next power of
  ten (T-4110)" (negative dollars mirror the positive ones; a power of ten returns the dollar).
- **Excel kept as the target (decision 0300):** 27 points are more than 1e-4 relative off the math, all bit-exact in
  the engine. EFFECT A85, A87, A92, A93, A95, A99, A102, A104, A110, A113, A116, A119, A120 and NOMINAL A85, A87, A89,
  A91, A94, A95, A97, A106, A108, A109, A115, A119, A120: a tiny rate lost in the double `1 + r/n` or `(1+e)^(1/n)`
  (0 where it rounds to 1; up to 61%, `NOMINAL(1.04206680307683E-09,7542681)`). RRI A163 (3.5E-4), where
  `EXP(x)-1` cancels near a growth of 0. Seven further CUMPRINC points (A148, A153, A156, A159, A170, A178, A179)
  have a math value below the double range (Excel 0), inside the absolute floor, not listed as off.
- Math sidecars: `rustsheet-accuracy math --function` over the seven functions for probes and probes-numeric: 25 new
  sidecars and additions to 10 existing ones, 4,596 cells of these functions (36 math domain errors), no existing
  cell changed. `make eval-math-check` prints "current".
- Overlap: T-4059 (todo, functions-financial) also asks for CUMPRINC and CUMIPMT `s1` tables; this task delivers them,
  so T-4059 is left with its `r4-T-3141` batches.
- Eval delta: none in the gated suites (tools, tables, sidecars, known failures and docs only).
