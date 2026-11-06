---
id: T-4105
title: "Gate A tables, discrete: BINOMDIST, NEGBINOMDIST, HYPGEOM.DIST, HYPGEOMDIST and POISSON"
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
BINOMDIST, NEGBINOMDIST, HYPGEOM.DIST, HYPGEOMDIST and POISSON each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0272 (T-3798) classified these functions as needing a table.
- A legacy name gets its own table that reuses the modern function's Definition (the default of T-3798: TINV already has
  its own table beside T.INV.2T). Where the alias's arguments differ (tails, a right tail, a cdf only), the shared
  Definition takes that into account rather than being copied (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Reuse `defs/discrete.py` (T-3553). HYPGEOM.DIST has no definition yet: add it beside the others.
- Sources (statistics.md): "HYPGEOM.DIST is not exact for small integer arguments", "HYPGEOM.DIST: per-call cap and
  lifting (T-1873)", "HYPGEOMDIST: a sample below the possible range is 0; a negative or huge count is #NUM! (T-2255)",
  "POISSON.DIST pmf: two closed forms (T-2055)", "POISSON at x 0 with a negative mean", "NEGBINOM.DIST pmf sits on the
  shifted binomial kernel (T-2513)".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/discrete.py`: BINOMDIST, NEGBINOMDIST and POISSON reuse the BINOM.DIST,
  NEGBINOM.DIST and POISSON.DIST definitions through one factory each (decision 0272); NEGBINOMDIST takes no cumulative
  flag (the pmf, T-1048). The modern tables regenerate byte for byte.
  - New HYPGEOM.DIST and HYPGEOMDIST (its pmf): the exact ratio of binomials; the cdf sums away from the mode
    (`sum_away_from_mode` in `_common.py`, shared with BINOM.DIST.RANGE, whose table also regenerates byte for byte).
  - Domain from evidence: counts in [0, 2^31 - 2] (T-2255); population_s or number_sample above number_pop (T-1048).
    Outside the support the pmf is 0 and the cdf 0 or 1, the math's own values.
  - The four counts are drawn together. Cdf draws stay at a standard deviation <= 1000, under the engine's term-cap
    DEVIATION (T-1873, T-1896), and under 2^17 summed terms: the excel-graph oracle cannot calculate
    `HYPGEOM.DIST(153800,1626691102,202463,2147483646,TRUE)`, found by bisecting the table (recorded in statistics.md).
  - POISSON.DIST's cdf has no math value past a count of 1E12 (`NoValueError`): mpmath sums x terms, and the probes'
    `POISSON(1E16,1E16,TRUE)` would have taken hours. No table point is there.
  - Tests: tools/tests/test_accuracy_hypgeom.py (Microsoft's examples, the semantics entries' exact values, domains,
    draws, the legacy names).
- Tables `eval/accuracy/tables/{BINOMDIST,NEGBINOMDIST,HYPGEOM.DIST,HYPGEOMDIST,POISSON}/s1`: 197, 200, 200, 200 and 120
  points (3 BINOMDIST points dropped: mpmath's incomplete beta did not converge), filled from excel-graph.
- `rustsheet-eval accuracy`: 0 failing. `missing tables` 121 to 116 (on main as merged).
  - BINOMDIST, NEGBINOMDIST and POISSON: every point inside the envelope.
  - HYPGEOM.DIST: 164 inside. HYPGEOMDIST: 160 inside.
  - 76 known failures added, all owned by the new T-4502 (engine bugs; Excel matches the math):
    - 66 pmf points: past the double-double walk the engine's statrs closed form overflows to `#NUM!`.
    - 4 points at the empty population: Excel gives 1, the engine `#NUM!`.
    - 6 cdf points: the engine is up to 1.2E-6 off.
- Engine bit-exact with Excel on the numeric points, and errors equal:

  | Table | Bit-exact | Errors equal |
  |---|---|---|
  | BINOMDIST | 95 of 158 | 39 of 39 |
  | NEGBINOMDIST | 78 of 188 | 12 of 12 |
  | POISSON | 86 of 101 | 19 of 19 |
  | HYPGEOM.DIST | 48 of 183 | 17 of 17 |
  | HYPGEOMDIST | 35 of 190 | 10 of 10 |

  Every Excel error is a domain error of the definitions.
- Excel more than 1e-4 relative off the math, Excel kept as the target (decision 0300): one point,
  `HYPGEOM.DIST(41966,46221,65797,74726,FALSE)`, which is 0 in Excel against 8.1E-186. It is recorded together with the
  empty population (`HYPGEOMDIST(0,0,0,0)` is 1, a new quirk) in statistics.md "HYPGEOM.DIST of an empty population is
  1, and a deep-tail pmf can be 0 (T-4105)".
- Math sidecars: `rustsheet-accuracy math --function` for the five functions over probes and probes-numeric.
  `make eval-math-check` is current.
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
