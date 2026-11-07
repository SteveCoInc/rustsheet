---
id: T-4749
title: "Probe Excel on the API-filter export's empty filters (every data row hidden) and blank=1 forms (T-4363 review follow-up)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the two API-filter export forms T-4363's upload did not cover: `<filters blank="1">` (a visible
blank data row) and an empty `<filters></filters>` (every data row hidden).

## Context
- Found by T-4363's excel-fidelity review (PR #3795, MINOR). T-4363 exports an API-filtered table with no kept
  criterion as a values filter on its first column listing the visible rows' text (`api_filter_column`,
  crates/rustsheet-xlsx/src/export/tables.rs); an excel-graph upload of our export with values "a" and "b" read as
  filtered (xlsx-storage.md "Table parts in the export"). The `blank="1"` form and the empty form were not uploaded;
  our import reads both as a criterion (`has_active_criterion`).
- Method: export a model with `Txn::add_table(.., filtered: true)` and hidden rows, then
  `cd tools && uv run rustsheet-oracle recalc -b excel-graph <file.xlsx>` (as T-4363 did); read SUBTOTAL 9/109 and
  3/103 over the table.

## Acceptance criteria
- [x] Both forms uploaded; whether Excel opens them without repair and reads them as filtered is recorded in
      xlsx-storage.md with the input sha256.
- [x] If Excel reads either as not filtered or refuses it, the export writes a form it reads as filtered (or a task
      is filed for it).
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters.

## Outcome
- Probed (excel-graph, 2026-11-07, eight uploads; sha256 of each in xlsx-storage.md "Table parts in the export"):
  Excel opens our `<filters blank="1">` exports (a blank beside values, and blank alone) and reads them as filtered,
  as ours. It reads the empty `<filters></filters>` our export wrote for a table with every data row hidden as **no
  criterion** (SUBTOTAL 9 21, 109 0), where we read 0, and the same for `<filters/>` and `<filters blank="0"/>`;
  `<filter val=""/>` makes it refuse the file; a values filter of the hidden rows' text and the custom filter
  "equals a and does not equal a" read as filtered.
- Export: an API-filtered table with no visible data row is written with that custom filter, which keeps no row if
  re-applied (`export::tables::api_filter_column`; test
  `a_filtered_table_with_every_row_hidden_exports_a_filter_that_keeps_none`).
- Import: a `<filters>` with no element inside and no true `blank` is no criterion (`is_criterion`, also in the
  dangling-color re-decision), as Excel reads it; unit cases in `only_a_criterion_inside_the_table_filters` and
  `dangling_color_filter_dxf_is_no_criterion`.
- The same rule, one definition (`FilterCriterion::is_empty_values`), in the import's dangling-color re-decision and
  in the engine's column edits (`kept_filter_inside`: an empty values filter is no kept criterion, unit test
  `an_empty_values_filter_is_no_kept_criterion`), and in the eval adapter (`filters_hidden_rows`: spec2xlsx writes a
  values filter with no visible data row as `<filters></filters>`).
- Probe case probes/t4749-empty-values-filter (12 cells, excel-graph): every data row of a `"filter": true` table
  hidden reads as not filtered, as the unfiltered control; 12/12 pass. The `blank="1"` forms cannot be written by
  spec2xlsx (a blank first-column cell in a filtered table is refused), so they stay as uploads.
- Docs: xlsx-storage.md ("Table parts in the export") and numbers.md (which filter makes a table filtered).
- Eval delta: probes +1 case (12 cells, all pass); every other suite unchanged (fixed 0, regressions 0).
