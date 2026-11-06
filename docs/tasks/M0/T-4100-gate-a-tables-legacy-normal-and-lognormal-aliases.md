---
id: T-4100
title: "Gate A tables, legacy normal and lognormal aliases: NORMDIST, NORMINV, NORMSDIST, NORMSINV, LOGNORMDIST, LOGINV and STANDARDIZE"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3783, T-3785]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
NORMDIST, NORMINV, NORMSDIST, NORMSINV, LOGNORMDIST, LOGINV and STANDARDIZE each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse T-3783's normal and T-3785's lognormal definitions (`defs/erf.py` and their modules). LOGNORMDIST is the cdf
  only. STANDARDIZE is the closed form (x - mean)/sd.
- Sources (statistics.md): "Legacy distribution names sweep (T-1047)", "Legacy distribution names sweep, second family
  (T-1048)", "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)", "SKEW, SKEW.P,
  KURT, HARMEAN, STANDARDIZE, SUMX* and MDETERM edge pools (T-1623)".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- Definitions reuse the modern ones (directive 3): in `defs/normal.py`, NORM.DIST, NORM.INV and NORM.S.INV now come
  from one factory each that also registers NORMDIST, NORMINV and NORMSINV; NORMSDIST is NORM.S.DIST's value with
  cumulative TRUE. In `defs/continuous.py`, LOGNORM.INV's factory also registers LOGINV, and LOGNORMDIST is
  LOGNORM.DIST's value and draws with the flag fixed to TRUE (shared `_LOGNORM_X`/`_LOGNORM_MEAN` params).
  STANDARDIZE (`normal.py`) is (x - mean)/sd.
  - Error regions are only the modern names' evidenced ones: statistics.md T-1048 shows every paired legacy/modern cell
    identical, value and error, so no legacy difference is encoded. STANDARDIZE's standard_dev <= 0 is `#NUM!`
    (statistics.md T-2255, T-2436). A STANDARDIZE difference or quotient past the double range, or a subnormal
    quotient, is unprobed: NoValueError (redrawn).
  - Run over the literal one-call cells of these functions in eval/suites (492 cells): no math domain error where
    Excel gives a number, nor the reverse.
  - Tests in tools/tests/test_accuracy_legacy_normal.py (Microsoft's examples, the t1048 probe cells, T-1623's
    `STANDARDIZE(5,3,1E-300)`, the error domains, the shared values).
- Tables `eval/accuracy/tables/{NORMDIST,NORMINV,NORMSDIST,NORMSINV,LOGNORMDIST,LOGINV,STANDARDIZE}/s1`: 164, 200,
  103, 120, 200, 200 and 195 points (NORMDIST and NORMSDIST drop deep-underflow points mpmath cannot converge, as the
  NORM.DIST and NORM.S.DIST tables do), filled from excel-graph with 0 failures.
- `rustsheet-eval accuracy`: 1,182 new points, all inside the envelope (1,174 on both sides, 8 through the math only:
  4 NORMINV, 3 NORMSINV, 1 LOGINV). The engine is bit-exact with Excel on 1,098 (all of LOGNORMDIST, STANDARDIZE,
  NORMDIST and NORMSDIST; NORMINV 177 of 200, LOGINV 169 of 200, NORMSINV 90 of 120). 0 failing, 0 table defects,
  0 known failures added. `missing tables` 102 to 95.
- **Excel kept as the target (decision 0300):** 14 points are more than 1e-4 relative off the math, all Excel's
  lower-tail flush to 0 where the math is a subnormal below the 1e-300 absolute floor (T-1040): `NORMSDIST(-37.625)`
  (A68, A70, A72, A74, A76, A78, A80, A82, A84; math 4.19E-310) and LOGNORMDIST A77, A116, A118, A134, A139 (math
  4.28E-320 to 4.19E-310).
- Semantics: statistics.md new entry "STANDARDIZE is the double quotient (x - mean)/standard_dev; standard_dev <= 0 is
  #NUM! (T-4100)" (Excel equals the double quotient on all 185 numeric points; the sd 0 cells), and T-1048's entry
  gains the table evidence (the 44 legacy-table errors and the flush cells).
- Math sidecars: probes and probes-numeric regenerated with `--function` for the seven names, additive only (728 new
  cells, none changed); `make eval-math-check` current.
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
