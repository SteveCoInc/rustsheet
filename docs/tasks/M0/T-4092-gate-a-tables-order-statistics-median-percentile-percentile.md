---
id: T-4092
title: "Gate A tables, order statistics: MEDIAN, PERCENTILE, PERCENTILE.INC, PERCENTILE.EXC, QUARTILE, QUARTILE.INC and QUARTILE.EXC"
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
MEDIAN, PERCENTILE, PERCENTILE.INC, PERCENTILE.EXC, QUARTILE, QUARTILE.INC and QUARTILE.EXC each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (statistics.md): "MEDIAN of an even count is a + (b - a)/2", "PERCENTILE and PERCENTILE.INC interpolate from
  a one-based rank 1 + k(n-1)", "QUARTILE reports quart's error before empty data (T-2455)", "A single cell holding a
  boolean is no number in the PERCENTILE and QUARTILE data (T-2429)".
- AGGREGATE 16 to 19 already has these kernels in `defs/aggregate_array.py` (T-3693): reuse them.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- **Definitions:** `accuracy/defs/order_statistics.py` defines all seven functions over the literals' sorted doubles.
  - MEDIAN is the middle value, or the exact midpoint of the two middle values.
  - PERCENTILE and PERCENTILE.INC interpolate at the one-based rank 1 + k(n - 1), and PERCENTILE.EXC at k(n + 1).
  - QUARTILE and QUARTILE.INC read floor(quart) in 0-4 as the INC percentile at quart/4, and QUARTILE.EXC reads
    floor(quart) in 1-3 as the EXC percentile at quart/4.
  - Domain (Excel's `#NUM!`): k outside [0, 1], or outside (0, 1) with the rank outside [1, n] for EXC, and a quart
    outside its range. All 25 drawn error points are `#NUM!` in Excel too.
  - The floor of a fractional quart is evidenced: probes/stat-order-floor-batch A85, A88 and A89.
  - MEDIAN's even case where `b - a` overflows is unprobed, so such a point gets no value and is never drawn.
- **Reuse (directive 3):**
  - From AGGREGATE 16-19 (`defs/aggregate_array.py`, T-3693): the kernels `percentile_inc`, `percentile_exc` and
    `quart_k`, and the samplers `valid_point` and `near_bound_point`. These were made public. The two samplers were
    split out of `_point` and `_near_bound` with the same rng order, so a regenerated AGGREGATE table is
    byte-identical.
  - `near_bound_point` gained whole-quart branches for QUARTILE.INC and QUARTILE.EXC.
  - MEDIAN draws with SUM's variadic `draw_numbers` (T-3549, renamed from `_draw`).
  - Each function adds its own edge shapes: Microsoft's examples, aggregate-grid H43-H46, the exact-double EXC
    bounds, the error neighbours, and a tiny k that Excel's double rank loses.
  - Tests are in tools/tests/test_accuracy_order_statistics.py.
- **Tables:** `eval/accuracy/tables/{MEDIAN,PERCENTILE,PERCENTILE.INC,PERCENTILE.EXC,QUARTILE,QUARTILE.INC,QUARTILE.EXC}/s1`,
  120 points each, filled from excel-graph with 0 failures.
  - `rustsheet-eval accuracy`: all 840 points are inside the envelope (834 on both sides, 4 through Excel's 15
    digits only, 2 through the math only). 0 failing, 0 known failures added. `missing tables` went from 128 to 121.
  - The engine is bit-exact with Excel on 830 of 840. The other 10 are even-count MEDIANs, where the engine's
    `(a + b)/2` is the math and Excel's `a + (b - a)/2` is not (T-3017, open).
- **Excel more than 1E-4 off the math. Excel kept as the target (decision 0300):**
  - `MEDIAN(87.01,-87.0099999999999)` (MEDIAN A52): Excel 5.684341886080802E-14, math 4.973799150320701E-14 (14%).
  - `MEDIAN(0.9311,-0.931100000000003)` (A53): Excel -1.4432899320127035E-15, math -1.4988010832439613E-15 (3.7%).
  - PERCENTILE A76, PERCENTILE.INC A76 and A81: k = 1E-20 over data whose smallest value is 0. Excel gives 0, the
    math a tiny positive value (100%).
  - PERCENTILE A105 (extreme, k 0.3 over 71 values): Excel 1.05037392982959E-100, math 1.0450240239130319E-100
    (0.5%). Excel's double rank 1 + 0.3*70 is exactly 22.
  - All of these follow existing entries. The two MEDIAN cells and the 48/48 match of `a + (b - a)/2` over the
    table's even counts are added to statistics.md "MEDIAN of an even count is a + (b - a)/2". No new quirk.
- **Math sidecars:** SIDECARS_PENDING
- **Eval delta:** none (tools, tables, docs and sidecars only).
