---
id: T-4113
title: "Gate A tables, bonds: PRICE, YIELD, DURATION and MDURATION"
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
PRICE, YIELD, DURATION and MDURATION each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- YIELD is a solver over PRICE. Sources (financial.md): "PRICE and ODDFPRICE discount with Excel's `^`", "PRICE,
  DURATION and MDURATION sum order at a huge coupon rate and yield", "PRICE, YIELD, DURATION and MDURATION check order",
  "PRICE, DURATION and MDURATION at a huge yield or coupon", "YIELD with one coupon period left is closed form", "YIELD
  stops short of PRICE's root", "YIELD's #NUM! cells and its 100-step cap", "Yield and odd-coupon work cap", "Coupon dates
  keep the maturity's day". T-3722, T-3723 and T-3620 own engine cells of DURATION, PRICE and YIELD.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve. (`make check-fast` green locally; reviewers pending)

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/bonds.py`: one definition each for PRICE, YIELD, DURATION and MDURATION.
  The coupon schedule reuses the fit lab's `schedule` (fitlab/kernels/solvers.py: each coupon date the maturity moved
  back whole periods, month-end rule from the maturity) read on Excel's 1900 calendar; the day counts reuse
  `defs/yearfrac.py` (`us_30_360`, `eu_30_360`, `displayed_date`, `real_date`, and its date reader, now public as
  `whole_serial`); YIELD's root reuses `annuity.unique_rate` (T-3535). The math is exact on the double arguments:
  PRICE's documented sum (closed-form geometric sums) with DSC = E - A, its linear form with one coupon left; DURATION
  the Macaulay sum of the same flows, MDURATION over 1 + yld/f; YIELD the documented closed form with one period left
  and otherwise PRICE's unique root (`solver=True`, 300 points: PRICE falls strictly with the yield when DSC >= 0, so a
  positive price has one root; "YIELD stops short of PRICE's root" is the root choice, and a point with DSC < 0 gets no
  value).
  - Domain errors, each from a semantics entry: dates off the calendar, a settlement below 1 or not before the
    maturity, frequency not 1, 2 or 4, basis outside 0..4, a negative rate or yld, a price or redemption of 0 or below,
    a previous coupon before 1900-01-01, and for PRICE, DURATION and MDURATION a power `(1+yld/f)^(s+N-1)` or a dirty sum
    past the double range. No value within 1E-9 (in the logarithm) of the double range's top, for an infinite
    `100·rate` or DURATION weighted sum under a finite dirty sum, at PRICE's linear discount of 0, and for YIELD's
    one-period form from a settlement before 1900-03-01.
  - Pre-check before generating: the definitions over all 4,288 literal one-call cells of these functions in
    eval/suites (probes, probes-numeric, gen-m9 and the other spec suites): no math domain error where Excel has a
    number. PRICE, DURATION and MDURATION are within 1E-9 of Excel on every numeric cell; YIELD's 208 Excel `#NUM!`
    cells and 140 cells off by more than 1E-9 are its documented Newton (`#NUM!`, 100th iterates, stopping short).
  - Tests in tools/tests/test_accuracy_bonds.py (Microsoft's examples, the semantics entries' Excel values, the
    closed form, the evidenced errors and no-value regions, draws).
- Tables `eval/accuracy/tables/{PRICE,DURATION,MDURATION}/s1` (200 points each) and `YIELD/s1` (300), filled from
  excel-graph with 0 failures.
- `rustsheet-eval accuracy`: 896 of the 900 points inside the envelope, 0 failing, 0 table defects; `missing tables`
  102 to 98. The engine is bit-exact with Excel on 864 of 900 (PRICE 199, DURATION 189, MDURATION 190, YIELD 286).
  Four YIELD points are known failures: A230, A252 and A261 (extreme stratum, the engine's last digits outside Excel's 15,
  both short of the root) under T-2788, and A225 (Excel `#NUM!` at a price and redemption near 0.01, the engine
  converges to the root) under the new T-4523.
- Every Excel error on the tables matches a coded domain, except YIELD's Newton failures (Excel `#NUM!` where the math
  is the root): A160 and A164 (a price above 1E5 at redemption 100, documented) and A216, A225, A231, A278 and A284
  (a price and redemption far from par), recorded in financial.md.
- **Excel kept as the target (decisions 0300, 0301):** 2 points are more than 1e-4 relative off the math, both YIELD
  roots near 0 where Excel's Newton stops short: `YIELD(46176,47108,0.108576904443112,127.687110632994,100,1)` (A140,
  3.5E-2) and `YIELD(44098,44491,0.0822071318475585,108.860101988015,100,1,4)` (A184, 2.4E-2). The engine is bit-exact
  with Excel on both. No R3 marks.
- New quirks, recorded in financial.md: "PRICE, DURATION and MDURATION take DSC as E - A on every basis" (163 table
  points on bases 2 and 3 where E - A is not the actual days to the next coupon, Excel within 1.2E-13 of the math) and
  "YIELD is #NUM! at a price and redemption far from par".
- Math sidecars: `rustsheet-accuracy math --function` over the four functions, for probes and probes-numeric: 29 new
  sidecars (4,007 cells, 116 of them math domain errors) and 175 cells added to existing ones, with no existing cell
  changed or removed. `make eval-math-check` prints "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
