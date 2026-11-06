---
id: T-4169
title: "Data field number formats in rendered pivot cells"
milestone: M16
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4149]
---

## Goal
Rendered value cells carry the data field's number format, so their displayed text matches Excel's.

## Context
Reuse the formatter (directive 3); displayed text is scored by text_expect. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Displayed text of pivot cells is not in sidecars yet (T-4123 captures values only): this task adds a `Range.text` read of each pivot's area to the excel-graph fill, beside the `text` cells it reads today, and stores it in the sidecar's `pivots` entries.

## Acceptance criteria
- [x] Text probes pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Conditional formats.

## Outcome
Rendered value cells now carry their data field's number format, and their displayed text matches Excel's.
- Oracle: the excel-graph fill reads `Range.text` of the used range of each sheet that holds a pivot, after
  the recalculation (`GraphBackend._read_used_texts`, `package.pivot_sheets`). It crops that text to each
  pivot's area in the sidecar's `pivots` entries (`text`, in `snapshot.pivot_areas`).
- Eval: `OraclePivot.text` adds a text_expect expectation for each cell (`text_expect::with_pivot_text`).
- Engine: `RenderedCell::number_format` is the data field's `numFmtId` on every value cell, including
  subtotals and grand totals. `apply_pivot_refresh` sets that format on the cell
  (`Model::set_pivot_number_format`, reusing `get_num_fmt` and the style pool) and keeps the rest of the
  cell's style. Built-in id 14 is now `m/d/yyyy` (Excel en-US), and new custom ids start at 164, as in
  Excel and XlsxWriter (number-formats.md).
- Probe: probes/t4169-pivot-number-formats, two pivots with formats 7, 10, 14, 164, 0, none and 4. All 84
  value cells and 76 text cells pass; eval known failures did not change (REGRESSIONS 0).
- Findings: pivots.md "Value cells show their data field's number format" and number-formats.md
  "Built-in id 14 is the short date; custom ids start at 164". UNVERIFIED readings (cells that stop being
  value cells, preserveFormatting, showDataAs and axis-field formats) are owned by T-4717.
