---
id: T-4098
title: "Gate A tables, CHISQ.TEST, CHITEST, Z.TEST, ZTEST and PROB"
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
CHISQ.TEST, CHITEST, Z.TEST, ZTEST and PROB each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0271 (T-3799) classified these functions as needing a table.
- The generator's one-row array literals (T-3586) and 2-D literals (T-3686) drive the data arguments. Variadic data
  follows T-3549's SUM and AVERAGE definitions (the draw of a variable-length array and the near-zero snap rule):
  reuse it, do not copy it (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Sources (statistics.md): "Z.TEST: an error array first; one data value is #DIV/0!; a lone non-number cell is #N/A
  (T-2256)", "PROB: an error data argument first, one value on each side is #DIV/0!, unequal counts #N/A (T-2256)",
  "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments".
- Reuse the chi-squared definitions (`defs/gamma.py`, T-3529) and the normal CDF (`defs/erf.py`). CHISQ.TEST takes two
  2-D arrays of the same shape (T-3686).

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/significance.py`: one definition each for CHISQ.TEST, CHITEST, Z.TEST, ZTEST
  and PROB, the legacy names registered by the modern builders (as T-4100).
  - CHISQ.TEST: the exact statistic over 2-D array literals, then CHISQ.DIST.RT's definition (`gamma_dist`) at df
    (rows - 1)(columns - 1), or cells - 1 for one row or one column (t1049-chisqtest A12, A29:A31).
  - Z.TEST: NORM.S.DIST's Phi(-z) (`normal.cdf`) with z from the exact mean and, without sigma, the exact sample
    standard deviation; Phi(-z) is 0 from z = 37.625 (normal.py `FLUSH`).
  - PROB: the exact sum of the probabilities in the limits.
  - Reused: `gamma_dist.CHISQ_DIST_RT`, `normal.cdf`/`FLUSH`, `aggregate.running_total`, `aggregate_array.array_of`,
    `dispersion.deviations_overflow`, and `paired_regression.constant_exact` (made public for this).
  - Domain errors, each from statistics.md and probes/t4098-tests-prob-domains (56 cells, filled on excel-graph for
    this task): CHISQ.TEST `#N/A` for unequal counts or one cell (a 1x1 literal too), `#DIV/0!` for an expected 0,
    `#NUM!` for a negative statistic or an overflowing squared difference; Z.TEST `#DIV/0!` for one value or constant
    data without sigma, `#NUM!` for sigma <= 0, overflowing squared deviations without sigma, or z past the double
    range above 0 with sigma given; PROB `#N/A` for unequal counts and `#NUM!` for a sum 1E-15 below 1 or 1E-14 above
    or further.
  - No math value where nothing evidences Excel: equal-count CHISQ.TEST arrays of different shapes when one is a table
    (T-2650), PROB sums between the probed bands, Z.TEST's inexact constants (`Z.TEST({0.1,0.1,0.1},0)` is 0 in Excel)
    and z past the range below 0 (Excel 0.5), overflowing sums, subnormal literals.
  - Before generating, the definitions ran over all 3,819 one-call cells of the five functions in eval/suites. 172
    fit them: 109 numbers, 57 domain errors with Excel's codes, 2 with no value (t4098 A49, A50). No math domain error
    where Excel has a number, and no math number where Excel has an error. CHISQ.TEST cells over ranges are skipped
    (a matrix argument, T-4025).
  - Tests in tools/tests/test_accuracy_significance.py: Microsoft's examples, the semantics entries' and the probe's
    Excel values, the evidenced domains, the unprobed points and the draws.
- Tables `eval/accuracy/tables/{CHISQ.TEST,CHITEST}/s1` have 120 points each and `{Z.TEST,ZTEST,PROB}/s1` 200 each (840
  in all), filled from excel-graph with 0 failures. Excel gives an error on 49 points and the math a domain error on the
  same 49, with the same codes.
- `rustsheet-eval accuracy`: all 840 points inside the envelope or known, 0 failing, 0 table defects. `missing tables`
  58 to 53. The engine is bit-exact with Excel on 805 of 840 (Z.TEST and ZTEST 400 of 400, PROB 198 of 200, CHISQ.TEST
  100 of 120, CHITEST 107 of 120; the 33 CHISQ.TEST/CHITEST points differ in the last digits, within the envelope).
  Known failures added: accuracy/PROB/s1 A113 and A125 (`PROB({1,2},{1.5,-0.5},2)`: Excel -0.5, the engine `#NUM!`),
  owned by T-4578, filed here with the probe's nine engine-off cells (PROB's per-probability check and sum tolerance,
  CHISQ.TEST's overflowing square, Z.TEST's z overflow).
- **Excel kept as the target (decision 0300):** none. Every Excel number on the tables is within 1.4E-10 relative of
  the math (88 points have both below the 1E-300 floor), and so is every numeric probe sidecar cell of these functions
  (at most 5.3E-15). Recorded in statistics.md.
- statistics.md: new entry "CHISQ.TEST, Z.TEST and PROB: Gate A domains (T-4098)" (PROB checks only the sum, its
  lopsided tolerance, CHISQ.TEST's 1x1 literal and overflow, Z.TEST's two-pass deviations, z overflow, inexact
  constants); the PROB entry's "a probability outside [0, 1] is `#NUM!`" corrected.
- Math sidecars: `rustsheet-accuracy math --function` over the five functions, for probes and probes-numeric. That adds
  probes/t4098-tests-prob-domains (48 cells, 34 domain errors, 2 skipped), probes-numeric/t1048-ztest (74) and
  t1049-ztest (16), and 7 cells added to stat-argument-modes. No existing cell changed. `make eval-math-check` prints
  "current".
- Eval delta: the new probe case (9 engine-off cells, owned by T-4578); none in the gated suites otherwise.
