---
id: T-4094
title: "Gate A tables, paired regression: FORECAST, FORECAST.LINEAR, INTERCEPT, SLOPE, RSQ, PEARSON and CORREL"
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
FORECAST, FORECAST.LINEAR, INTERCEPT, SLOPE, RSQ, PEARSON and CORREL each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (statistics.md): "Paired regression functions: single points, zero variance, text and large offsets",
  "CORREL, PEARSON and RSQ over arrays of different sizes, and their last digit", "RANK, FORECAST, PROB, Z.TEST and
  T.TEST: lifted scalar arguments", "A one-cell reference to an empty cell is #VALUE! in the paired functions and F.TEST
  (T-2255)".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve (design, security and Excel fidelity approve).

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/paired_regression.py`: one definition each for FORECAST, FORECAST.LINEAR,
  INTERCEPT, SLOPE, RSQ, PEARSON and CORREL. Each is the exact value on the literals' doubles, from exact rational
  means and sums about them: SLOPE = Sxy/Sxx, INTERCEPT = ȳ - SLOPE·x̄, FORECAST = ȳ + SLOPE·(x - x̄),
  r = Sxy/sqrt(Sxx·Syy), RSQ = r². The arrays reuse AGGREGATE's `array_of`; the sums of the values reuse `running_total`.
  - Domain errors, each from statistics.md: unequal lengths `#N/A`; one pair or a constant x `#DIV/0!`; a constant y
    `#DIV/0!` for the correlations only (SLOPE 0, INTERCEPT and FORECAST the mean); `#NUM!` where Excel's double sum about
    x's mean (or the sum of x) overflows, and for the correlations y's too (T-4047).
  - No math value where nothing evidences Excel's result: a constant series whose double mean is not its value
    (`CORREL({0.1,0.1,0.1},{1,2,3})` is 0 in Excel), a subnormal literal or deviation square or product, and for SLOPE,
    INTERCEPT and FORECAST a sum of y or an Sxy past the double range.
  - Before generating, the definitions were run over all 5,228 one-call cells of these functions in eval/suites. 248 fit
    them: 116 numbers, 126 domain errors, 6 with no value (the 0.1 constants above). Where Excel has a number, no math
    domain error; where Excel has an error, no math number.
  - Tests in tools/tests/test_accuracy_paired_regression.py: Microsoft's examples, the semantics entries' Excel values,
    the evidenced domains, the unprobed points and the draws.
- Tables `eval/accuracy/tables/{SLOPE,INTERCEPT,RSQ,PEARSON,CORREL}/s1` have 120 points each and
  `{FORECAST,FORECAST.LINEAR}/s1` 200 each (1,000 in all), filled from excel-graph with 0 failures. Excel gives an error
  on 85 points and the math a domain error on the same 85, with the same error codes.
- `rustsheet-eval accuracy`: all 1,000 points inside the envelope (985 on both sides, 15 through Excel's 15 digits
  only), 0 failing, 0 table defects, 0 known failures added. `missing tables` 102 to 95. The engine is bit-exact with
  Excel on 998 of 1,000. The other two are FORECAST A67 (about 30 ulps, a cancelling line) and A79 (1 ulp), both within
  15 digits of Excel. Excel's bits there follow x87 sums of the values and a double-rounded slope division. That model
  fits all 597 numeric FORECAST, FORECAST.LINEAR, SLOPE and INTERCEPT points; the engine's plain doubles fit 595. This is
  recorded in statistics.md "Paired regression functions" (T-4094 paragraph), and follow-up T-4550 owns the engine change
  (deferred to phase 1b like T-3257, since both points are within 15 digits).
- **Excel kept as the target (decisions 0300, 0301):** 9 points are more than 1e-4 relative off the math: INTERCEPT A109,
  A110, A112, A116, A117, A119, A120, FORECAST A192 and FORECAST.LINEAR A184. All are log-uniform draws over 1E-150 to
  1E+150, where Excel's ȳ - SLOPE·x̄ and a + b·x cancel. For example, INTERCEPT A109 is 0 against an exact 3.70E+67, and
  FORECAST.LINEAR A184 is -4.21E+65 against -7.85E-40. The engine is bit-exact with Excel on all nine. CORREL, PEARSON,
  RSQ and SLOPE are within 1e-9 of the math on every numeric table point. Also recorded in statistics.md.
  Five eval cells are kept the same way: probes/t4047-paired-overflow J1 and J7 (CORREL, PEARSON, 1.1e-4), J6 (RSQ,
  2.3e-4), and J3 and J5 (INTERCEPT, FORECAST, 6.7e-4). Their data are a few ulps apart near 1E+160, where Excel's
  double mean decides. The engine matches Excel on all five.
- Math sidecars: `rustsheet-accuracy math --function` over the seven functions, for probes and probes-numeric. That gives
  5 new probes sidecars (204 cells, 104 of them math domain errors) and 14 cells added to stat-argument-modes. No cell
  of probes-numeric fits, and no existing cell changed. `make eval-math-check` prints "current". The new sidecars give
  probes/t4047-paired-overflow J1:J3 and J5:J7 math values 1.4E-5 to 6.7E-4 relative off Excel (J2, SLOPE, is 1.4E-5;
  the other five are listed above).
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
