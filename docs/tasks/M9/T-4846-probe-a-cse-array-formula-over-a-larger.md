---
id: T-4846
title: "Probe a CSE array formula over a larger range in-repo: 1x1, row, column, 2-D #N/A fill and IFERROR (T-0950 follow-up)"
milestone: M9
lane: engine
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: [T-4227]
---

## Goal
An in-repo probe case, filled by the Excel oracle, pins the CSE fill over a larger range. The PR gate then catches a
regression in it without the external poi and corpus-bench suites, which only nightly fetches.

## Context
- T-0950 (PR #3889) implemented arrays.md "Legacy (CSE) array formula over a larger range" in
  `Model::set_cells_with_result` (model.rs, CSE branch, `arithmetic::bcast_get`). Its evidence is all Excel-cached and
  external: poi/MatrixFormulaEvalTestData.xlsx H46:J136 and corpus-bench/13894 F20:F25. reviewer-excel-fidelity
  (MINOR) asked for in-repo coverage.
- Check first whether the case builder (tools/, spec2xlsx) can write a legacy array formula (`<f t="array" ref=..>`).
  If it cannot, record that and file the builder change.

## Acceptance criteria
- [ ] A probes case with an oracle sidecar (excel-graph) holds CSE formulas over larger ranges: a 1x1 result over 3x3,
      a one-row result over 3x3, a one-column result over 3x3, a 2x2 result over 3x3, and an IFERROR-wrapped one-column
      result over a taller range. Predictions are committed before the fill.
- [ ] Every cell passes (or each mismatch is recorded in arrays.md and owned by a task).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array spills (`#SPILL!`).

## Blocked because
The case builder cannot write a multi-cell legacy array formula: `spec2xlsx.py` only calls
`write_dynamic_array_formula(row, col, row, col, ..)` (single anchor) or `write_formula` for `legacy_formulas`
(single cell), and `rg array_formulas` finds nothing in `tools/` or `crates/rustsheet-eval`. The builder change is
already filed as T-4227 (`array_formulas` key, spec2xlsx writer plus engine-adapter entry). Unblock this task once
T-4227 lands, then write the probe case (1x1, row, column, 2x2 and IFERROR-wrapped column results over 3x3 / taller
ranges), predictions first, and run `rustsheet-oracle fill -b excel-graph --suite probes` (status: excel-graph OK).
