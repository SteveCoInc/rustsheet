---
id: T-4096
title: "Gate A tables, array regression: LINEST, LOGEST, TREND and GROWTH"
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
LINEST, LOGEST, TREND and GROWTH each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- They return arrays, so each point reads one element through INDEX (for example `INDEX(LINEST(y,x),1,1)`); record
  the element in the point's formula. A multi-column x uses T-3686's 2-D literals.
- Sources (statistics.md): "LINEST and LOGEST: shapes, one point, collinear columns, exact fits and check order
  (T-1741)", "LINEST, LOGEST, GROWTH: probe findings", "LINEST, LOGEST and GROWTH in the gen-m9 sweep (T-2255)",
  "LINEST, LOGEST and GROWTH: cells left after T-1741 (T-1742)", "TREND: defaults, shapes, single points and error
  inputs", "LINEST and LOGEST report error values first (T-2455)".

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
- `tools/src/rustsheet_tools/accuracy/defs/array_regression.py`: LINEST, LOGEST, TREND and GROWTH, indexed
  (`=INDEX(FUNC(..),row,column)`) with const and stats (LINEST, LOGEST) or new_x and const (TREND, GROWTH) always
  written. The math is the exact least-squares fit: the normal equations in rational arithmetic on the literals'
  doubles, LOGEST and GROWTH on ln y taken 60 digits above the working precision. It reuses MINVERSE's exact
  `inverse` and INDEX parameters (`matrix.row_column`, `index_of`), AGGREGATE's `array_of` and the paired draws'
  `offset_series` and `forecast_x` (made public, no other change). Errors are domain rules from statistics.md
  (unequal shapes `#REF!`, non-positive y `#NUM!`, the stats block's `#N/A` cells). Not claimed (NoValueError):
  designs within a relative 1E-7 of collinear (Excel's column drop is T-2569's), the residual cells of an exact fit or
  of one with no degree of freedom left, r^2 of a constant y. T-3549's near-zero snap is not applied (no evidence
  that TREND's final addition snaps; no table point needed it).
- Elements covered: LINEST's and LOGEST's whole result, the coefficient row and all of the stats block (rows 2-5),
  for one to four x variables (a row known_y; a k-row known_x is k variables); every element of TREND's and GROWTH's
  result (any new_x shape for one variable, k rows for k variables). A column known_y is not drawn (the generator's
  arrays are one-row literals); it only transposes the shapes.
- Unit tests: tools/tests/test_accuracy_array_regression.py (Microsoft's LINEST, LOGEST, TREND and GROWTH examples,
  the statistics.md values, a cross-check of the one-variable stats against SLOPE, INTERCEPT, RSQ and STEYX, the
  domain rules, the draws).
- Tables (excel-graph, 2026-11-06): LINEST/s1 200 points (8 errors), LOGEST/s1 199 (17), TREND/s1 200 (3),
  GROWTH/s1 200 (9). `rustsheet-eval accuracy`: 0 failing, 0 table defects; `missing tables` 43 to 39.
  accuracy: 35760 points in 212 tables (pending 0): 35355 in envelope (Excel 15 digits only 1570, math only 907),
  405 known failures (..., T-2006 21, ..., T-4590 1), 0 failing, 0 table defects; missing tables 39.
- Known failures (22): 21 to T-2006 (the engine's normal-equations solve on ill-conditioned designs, 2E-9 to 20
  relative off where Excel is within 2E-8 of the math; TREND A193 `#NUM!`): LINEST A85, A87, A93, A98, A100, A102,
  A103, A104, A110, A129, A187; LOGEST A81, A82, A86, A98, A103, A110; TREND A94, A98, A193; GROWTH A97. Those
  points are visibly off, so T-2006 moves from `defer: phase-1b` to `campaign: accuracy` as its Deferred section says.
  1 to the new T-4590: GROWTH A104, Excel `#NUM!` where the prediction is a number (the ln fit's intercept e^1289.95
  overflows; unexplained).
- R3 candidates (Excel more than 1E-4 off the math; kept as the target under decision 0300, not marked): LINEST s1
  A197 (ss_reg 0 against 1.19E-52, the cancellation of ss_tot 3.9E+41; ours 0 as Excel). Next closest: LINEST A184
  8.5E-5, LOGEST A98 3E-6, LOGEST A82 8E-7; everything else within 2E-8.
- Found: one point without a constant term is a fit through the origin (TREND A140 and four more, GROWTH A123, A125),
  which settles the TREND entry's UNVERIFIED `TREND(5,2,4,FALSE)`; a GROWTH slope whose e^coefficient underflows is no
  error (A119, A127). statistics.md "LINEST, LOGEST, TREND and GROWTH: Gate A tables (T-4096)".
- Math sidecars: no probes or probes-numeric cell qualifies (each LINEST, LOGEST, TREND or GROWTH cell omits const,
  stats or new_x, reads a range, or is not one INDEX of literals), so `rustsheet-accuracy math` wrote none and
  `--check` says current.

