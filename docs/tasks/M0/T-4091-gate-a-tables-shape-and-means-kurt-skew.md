---
id: T-4091
title: "Gate A tables, shape and means: KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN, AVERAGEA and TRIMMEAN"
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
KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN, AVERAGEA and TRIMMEAN each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (statistics.md): "SKEW, SKEW.P, KURT, HARMEAN, STANDARDIZE, SUMX* and MDETERM edge pools (T-1623)", "SKEW.P of
  two numbers is #DIV/0!", "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)",
  "AVERAGEA, MAXA and MINA over an array", "TRIMMEAN: an error percent before data with no number (T-1742)",
  "TRIMMEAN of a typed scalar (T-3781)"; numbers.md "GEOMEAN root rounding", "GEOMEAN is #NUM! when the product of
  significands overflows", "GEOMEAN in-range root"; coercion.md "AVERAGE, MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT,
  STDEV/VAR, COUNT value arguments".

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
- **Definitions:** `accuracy/defs/shape_means.py` defines all seven functions over the literals' doubles.
  - SKEW, SKEW.P and KURT are Microsoft's formulas on the exact central sums S_2, S_3 and S_4. They reuse the
    dispersion module's DEVSQ kernel, generalized to `deviation_power_sum(xs, p)` (`squared_deviations` is p = 2).
  - GEOMEAN is the exact n-th root of the exact product, and HARMEAN the exact n / sum(1/x).
  - AVERAGEA over numbers is AVERAGE's math (T-3549), with its near-zero snap. `_average` became the public
    `average_value` for this.
  - TRIMMEAN drops floor(n * percent / 2) values from each end and is the exact mean of the rest.
  - Domain (Excel's error): SKEW and SKEW.P of fewer than 3 numbers, and KURT of fewer than 4, are `#DIV/0!`.
    Equal numbers are `#DIV/0!` where Excel's double sum of them is exact. Overflowing squared deviations are
    `#DIV/0!` (T-2432). GEOMEAN and HARMEAN with a number at or below 0 are `#NUM!`. TRIMMEAN with a percent outside
    [0, 1) is `#NUM!`.
  - Unprobed regions get no value (NoValueError), so the draws never keep them:
    - equal numbers whose double sum is inexact;
    - an overflowing sum of the numbers in the three moments;
    - an overflowing reciprocal or reciprocal sum in HARMEAN;
    - an overflowing running total in AVERAGEA;
    - a TRIMMEAN point where the double n * percent and the exact one floor to different trim counts;
    - an overflowing sum of TRIMMEAN's kept values.
  - GEOMEAN's significand-product `#NUM!` and its exponent wrap need more than 1000 numbers, beyond a call's 255
    arguments, so they are not in the definition.
- **Reuse (directive 3):**
  - The moments draw with the dispersion draws (T-4089), `_draws` made public as `variance_draws`, plus Microsoft's
    example in the edge stratum. SKEW.P (`_xlfn`) takes at most 254 numbers, as T-4089 found.
  - GEOMEAN and HARMEAN draw the magnitudes of the same draws, with their own edge shapes: Microsoft's example,
    exact roots, a 0 or a negative number, and products beyond the double range.
  - AVERAGEA draws with SUM's `draw_numbers`.
  - TRIMMEAN draws with PERCENTILE.INC's `valid_point` and `near_bound_point` (T-4092), reading k as the percent.
  - Domain checks reuse `running_total`, `overflow` and `deviations_overflow`.
  - Tests are in tools/tests/test_accuracy_shape_means.py. They cover Microsoft's examples (KURT -0.151799637,
    SKEW 0.359543071, SKEW.P 0.303193339, GEOMEAN 5.476986969, HARMEAN 5.028375962, TRIMMEAN 3.777777778) and the
    semantics values (SKEW.P(1,2,4), KURT(1,1,1,2) = 4, `AVERAGEA(0.1,0.2,-0.3)` = 0, and others).
  - Before generating, the definitions were run over every one-call cell of the seven functions in probes (125),
    probes-numeric (360) and gen-m9 (128). No cell has a math domain error where Excel gives a number, or the
    reverse.
- **Tables:** `eval/accuracy/tables/{KURT,SKEW,SKEW.P,GEOMEAN,HARMEAN,AVERAGEA,TRIMMEAN}/s1`, 120 points each, filled
  from excel-graph with 0 failures.
  - `rustsheet-eval accuracy`: all 840 points are inside the envelope (803 on both sides, 32 through Excel's 15
    digits only, 5 through the math only). 0 failing, 0 table defects, 0 known failures added. `missing tables` went
    from 88 to 81.
  - The engine is bit-exact with Excel on 777 of 840: AVERAGEA 120, HARMEAN 120, SKEW 120, SKEW.P 119, GEOMEAN 110,
    KURT 94 and TRIMMEAN 94. The other 63 are last-digit differences inside the envelope.
  - All 49 error points are the coded domain's error in Excel too. New entry: statistics.md "KURT, SKEW, SKEW.P,
    GEOMEAN, HARMEAN and TRIMMEAN domain cells of the Gate A tables (T-4091)". It states each rule as far as its
    cells show it and marks the inexact-sum equal numbers and the TRIMMEAN trim-count boundary UNVERIFIED. Every
    definition cites it.
  - Worst Excel-against-math gaps outside AVERAGEA: SKEW 7.2E-6 (A59, offset data around 1726160.3), SKEW.P 1.2E-6
    (A51), KURT 1.8E-7 (A65), and 2E-15 or less for GEOMEAN, HARMEAN and TRIMMEAN.
- **Excel kept as the target (decision 0300):** Excel is more than 1E-4 relative off the math on 14 AVERAGEA points.
  Each is a near-cancelling sum from SUM's tail draw, where Excel's double sum keeps a residue the exact sum of the
  literals does not have. The engine matches Excel on all 14.
  - AVERAGEA/s1 A49 (3.1E-4), A51 (6.4E-3), A53 (1.5E-2), A56 (4.6E-2), A57 (3.0E-2), A58 (3.8E-2), A59 (2.8E-2),
    A60 (5.5E-4), A62 (4.3E-2), A64 (4.5E-2), A65 (5.9E-3) and A66 (2.5E-2).
  - AVERAGEA/s1 A52 `AVERAGEA(0.1089,0.06369,0.2141,-8579,8578.61331,0)` and A54: Excel is 0 and the math is
    1.06E-13 and 3.24E-14. Excel's twice-rounded running total of the numbers before the trailing 0 is already
    exactly 0, so the residue is lost in the additions.
  - Probe sidecar cell probes/subnormal-flush-edges A8, `AVERAGEA(3E-308,-2.9E-308)`: Excel 0, math 5E-310. This is
    a subnormal result, which Excel flushes to 0 (numbers.md "Subnormal results become 0").
- **Math sidecars:** made with `rustsheet-accuracy math --function` for the seven functions, in probes and
  probes-numeric. There are 485 new math cells: 26 in nine existing sidecars and 459 in eight new ones
  (probes-numeric t3502-geomean-fresh and t3502-geomean-postfreeze; probes geomean-mixed-magnitude,
  geomean-root-refinement, geomean-same-product, geomean-single-and-copies, skew-p-bound and
  subtotal-reference-operands-d).
  - The change is additive: no existing entry changed.
  - Every new cell agrees with Excel's oracle value within 1E-4 or as an error on both sides, except the subnormal
    A8 above.
  - `rustsheet-accuracy math --check` reports current.
- **Eval delta:** none (tools, tables, docs and sidecars only). `scripts/check_duplication.py` reports 2836 (baseline
  2876).
