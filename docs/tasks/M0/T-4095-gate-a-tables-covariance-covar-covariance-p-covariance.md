---
id: T-4095
title: "Gate A tables, covariance: COVAR, COVARIANCE.P, COVARIANCE.S and STEYX"
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
COVAR, COVARIANCE.P, COVARIANCE.S and STEYX each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (statistics.md): "COVARIANCE: a boolean scalar is one value (T-2436)", "Paired regression functions: single
  points, zero variance, text and large offsets". Share the paired-data sampler with T-4094 (directive 3).

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
- `tools/src/rustsheet_tools/accuracy/defs/paired_regression.py` (T-4094's module, extended, not copied): one definition
  each for COVAR, COVARIANCE.P, COVARIANCE.S and STEYX on the same exact-fraction `moments`, `_series` and
  `_constant_exact`. COVAR and COVARIANCE.P = Sxy/n, COVARIANCE.S = Sxy/(n-1), STEYX = sqrt((Syy - Sxy²/Sxx)/(n-2)).
  `Moments` gains the sum-of-x overflow flag and Excel's double Sxx, Syy, Sxy; the draws gain a minimum pair count and
  a tail noise floor (defaults unchanged, so T-4094's seven tables still verify with 0 differences).
  - Domain errors, each from statistics.md: unequal lengths `#N/A`; COVARIANCE.S of one pair `#DIV/0!` (COVAR and
    COVARIANCE.P 0; t2436/computed-scalar A29, A30); STEYX `#NUM!` for x's overflow before its fewer-than-three-pairs
    `#DIV/0!`, `#DIV/0!` for a constant x, `#NUM!` for y's sum of squares overflowing with four or more pairs (T-4047)
    and wherever Excel's double Syy - Sxy*Sxy/Sxx is below zero (steyx-exact-fit; Sxy² overflowing, T-4047 J4).
  - No math value where nothing evidences Excel's result: inexact constants (as T-4094), for the covariances a sum of
    the values or Sxy past the double range, for STEYX y's overflow with two or three pairs or a constant x, and
    subnormal intermediates.
  - Before generating, the definitions were run over all 2,844 one-call cells of these functions in eval/suites. 95
    fit them: 54 numbers, 41 domain errors, none without a value. Where Excel has a number, no math domain error; where
    Excel has an error, no math number (all 14 probes/steyx-exact-fit cells included, `#NUM!` and 0 alike).
  - Tests in tools/tests/test_accuracy_covariance.py: Microsoft's examples, the eval cells' Excel values, the
    evidenced domains, the unprobed points and the draws.
- Tables `eval/accuracy/tables/{COVAR,COVARIANCE.P,COVARIANCE.S,STEYX}/s1`, 120 points each (480), filled from
  excel-graph with 0 failures. Excel gives an error on 33 points (COVARIANCE.P 4, COVARIANCE.S 7, STEYX 22) and the
  math a domain error on the same 33, with the same codes.
- `rustsheet-eval accuracy`: all 480 points inside the envelope (470 on both sides, 10 STEYX tail points through
  Excel's 15 digits only), 0 failing, 0 table defects, 0 known failures added. `missing tables` 76 to 72. The engine is
  bit-exact with Excel on all 480.
- New domain rule recorded (statistics.md "Paired regression functions", T-4095 paragraph): STEYX's residual-below-zero
  `#NUM!` is not only an exact-fit effect. Over 1E-150 to 1E+150 data, 7 of 12 log-uniform points are `#NUM!` where
  the exact value is a number (A109, A110, A113, A114, A115, A116, A118). Also recorded: the covariances stay finite
  when one argument's Σ(x-x̄)² overflows, and COVARIANCE.S of one pair is `#DIV/0!`.
- **Excel kept as the target (decision 0300):** 2 table points are more than 1e-4 relative off the math: STEYX A79
  (0 against 7.2E-17) and A84 (3.332000937312528E-09 against 3.2E-17), exact decimal lines whose residual is rounding.
  Among eval cells with math sidecars: probes/steyx-exact-fit A1, A2, A3, A4, A5, A7, A10, A11, A12, A14 and
  probes/correl-inexact-constant-overflow Z79. The engine matches Excel on all 13. COVAR, COVARIANCE.P and COVARIANCE.S
  are within 1e-9 of the math on every table point. Also recorded in statistics.md.
- Math sidecars: `rustsheet-accuracy math --function` over the four functions, for probes and probes-numeric. That
  gives 2 new probes sidecars (covariance-pairs-skip, steyx-exact-fit) and cells added to correl-inexact-constant-overflow,
  regression-pairs-b, stat-argument-modes and t4047-paired-overflow; no existing cell changed, none in probes-numeric.
  `make eval-math-check` prints "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
