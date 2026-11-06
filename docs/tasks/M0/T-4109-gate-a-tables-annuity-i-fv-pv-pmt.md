---
id: T-4109
title: "Gate A tables, annuity I: FV, PV, PMT, IPMT, PPMT, ISPMT and PDURATION"
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
FV, PV, PMT, IPMT, PPMT, ISPMT and PDURATION each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/annuity.py` (T-3535). Sources (financial.md): "Annuity evaluation order and `(1+r)^n`
  (PV/FV/PMT/NPER/IPMT/PPMT)", "PMT, IPMT and PPMT are the exact payment", "PMT is a discounted payment", "PPMT discounts
  with an atanh-series log", "IPMT cancels on fv", "IPMT follows Excel's own discount where it moves the value", "IPMT and
  PPMT split Excel's discounted payment", "ISPMT's operation order", "PMT when (1+rate)^nper overflows or underflows".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/annuity_values.py`: one definition each for FV, PV, PMT, IPMT, PPMT, ISPMT
  and PDURATION, the exact closed forms on the double arguments with `1+rate` exact. FV, PV and PMT reuse T-3535's
  `present_value`, and the draws reuse its `_annuity` (defs/annuity.py). No solver: none of these functions has a root.
  - Domain errors, each from a semantics entry or an eval cell: PV `#NUM!` at rate -1 with nper 0, `#DIV/0!` for a P
    below the normal range, `#NUM!` for a P past the double range; FV `#NUM!` past it; PMT `#NUM!` at rate -1 and below
    the normal range; IPMT and PPMT `#NUM!` for a period below 1 or at or past nper + 1 and where 1/P overflows (IPMT 0
    in period 1 of type 1); PDURATION `#NUM!` at a rate, pv or fv of 0 or below, `#DIV/0!` where 1 + rate is 1.
  - No math value where nothing evidences Excel's result (rate below -1, nper 0, P within 1E-6 in log2 of a cut, a
    subnormal P with a finite reciprocal for IPMT and PPMT). Before generating, the definitions were run over all 8,484
    literal one-call cells of these functions in eval/suites: no math domain error lies where Excel has a number.
  - Tests in tools/tests/test_accuracy_annuity_values.py (Microsoft's examples, the semantics entries' Excel values,
    the split against the exact balance's type-1 form, the evidenced domains, draws).
- Tables `eval/accuracy/tables/{FV,PV,PMT,IPMT,PPMT,ISPMT,PDURATION}/s1`, 200 points each (1,400), filled from
  excel-graph with 0 failures.
- `rustsheet-eval accuracy`: all 1,400 points inside the envelope (1,358 on both sides, 42 through Excel's 15 digits
  only: FV 15, PV 8, PDURATION 19), 0 failing, 0 known failures added. `missing tables` 128 to 121. The engine is
  bit-exact with Excel on 1,399 of 1,400; the other, `IPMT(0.127562322286594,143,229,-20238.0657453736,0,0)` (A6), is one
  ulp from Excel, which equals the math.
- New quirks, recorded in financial.md:
  - "IPMT and PPMT take a period up to nper + 1": a period between nper and nper + 1 is a value (PPMT A112, A118),
    and nper + 1 itself is `#NUM!`. The first generation coded "above nper" from memory, and the check flagged both points
    as table defects. The engine already had the bound (`period_payments`).
  - "PDURATION's domain": negative arguments are `#NUM!` (16 edge points), and Excel divides by the logarithm of the
    double `1+rate`, so a tiny rate is far off the math.
- **Excel kept as the target (decision 0300):** 9 points are more than 1e-4 relative off the math. Eight are PDURATION
  at rates from 2.3E-16 to 2.9E-13 (A82, A85, A89, A107, A112, A116, A120, A135; 2.2E-4 to 9.9E-2, the logarithm of
  the double `1+rate`). The ninth is `FV(2.69376404795075E-12,113,7.25542032231508,-930.933793192857,1)` (A87,
  2.2E-4), where P - 1 on the double `1+rate` cancels ("Annuity evaluation order"). The engine is bit-exact with
  Excel on all nine.
- Math sidecars: SIDECARS_PLACEHOLDER
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
