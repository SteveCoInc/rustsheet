---
id: T-4939
title: "A column delete that leaves a filter criterion re-applies the kept criteria to every data row: hide visible rows they reject, evaluate custom filters"
milestone: M8
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
After a column delete in the application that takes one criterion of a filtered table and leaves another, the
engine shows and hides the table's data rows the way Excel does. Excel re-applies the kept criteria to every data
row, so a visible row that a kept criterion rejects is hidden too. It compares a row by the text its cell displays,
ignoring case, and it evaluates a kept custom filter as well as a values filter.

## Context
- Evidence (T-4895, excel-graph 2026-11-07; docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden
  and filtered rows"):
  - probes/t4895-two-criteria-number-format-delete-one: Ext is formatted `0.00` and its kept filter lists the General
    text "10" .. "60". After the delete, **every** data row is hidden (SUBTOTAL(3,..) of each row is 0), including
    rows 2, 4 and 6, which were visible. The engine leaves rows 2-4, 6 and 7 visible: 14 of 18 cells off.
  - Uploads of patched workbooks, recorded with their sha256 in docs/research/t4895/predictions-uploads.tsv:
    - `blank-rejected`: a visible row whose kept cell is blank, under a filter without `blank="1"`, is hidden. The
      engine leaves it visible.
    - `custom`: a row hidden by the deleted criterion that a kept custom filter (equals a, or equals c) keeps is
      shown. The engine keeps it hidden.
    - `upper-case` (values "A"/"C" over cells a/c) and `blank-kept` match the engine.
- Code: `Model::rows_kept_by_criteria` and the reapply pass in `resize_tables`
  (crates/rustsheet-engine/src/table_edits.rs) only unhide rows, and only stored row records. `FilterCriterion::keeps_text`
  (table_types.rs) evaluates values filters only; `CustomFilter` keeps its operator as text.
- The eval adapter path (`Txn::filter_table_column`, spec `"criteria"`) gave the number-format case's rows 3 and 7 as
  kept. Check what text it stores for a formatted column: spec2xlsx lists the General text.

## Acceptance criteria
- [x] probes/t4895-two-criteria-number-format-delete-one passes (its baseline failure line goes).
- [x] First a positive control, probed with predictions committed before the fill: the same `0.00` column with its kept
      filter listing the displayed text ("10.00" .. "60.00"), deleting Cat. The display-text rule rests on one probe
      where nothing matched, so this control must show the rows the filter keeps before the engine implements it.
- [x] Unit tests, in the engine's table-edit tests, for the `blank-rejected` and `custom` uploads (rebuild them with
      docs/research/t4895/uploads.py) and for a rejected visible row with no stored row record.
- [x] numbers.md's rule is restated as Excel's re-apply, with the gaps removed.
- [x] `make check` green; reviewers approve. (`make check-fast` locally; CI runs `make check`.)

## Out of scope
- Kept top10, dynamic, date-group and color criteria (unprobed; they stay as they are).
- Re-applying filters after a value edit.

## Outcome
- Positive control first (docs/research/t4939/): `control.py` builds two uploads of T-4895's number-format table
  whose kept Ext filter lists the displayed text "10.00" .. "60.00". The predictions (`predictions.tsv`) were pushed
  before the excel-graph fill. Excel matched H_display: the control reads rows 3, 5 and 7 hidden, and the delete of
  Cat shows rows 2-4, 6 and 7 and hides row 5 (`uploads/*.oracle.json`). So the display-text rule rests on a
  positive result as well as the all-hidden one.
- Engine: `rows_decided_by_criteria` (table_edits.rs) replaces `rows_kept_by_criteria`. It re-applies the kept
  criteria to every data row of the table: it shows the rows they all keep and hides the rows one rejects. A rejected
  row with no row record gets a new hidden record (`Row::hidden_default`, now shared with `Worksheet::set_row_hidden`).
  A row whose verdict hangs on a criterion nothing evaluates keeps its state.
- `FilterCriterion::keeps_text` now also evaluates a custom filter (`CustomFilter::keeps_text`), for the evidenced
  form only: `equal`, or no operator, with a text operand that has no wildcard, combined by `and`. Every other
  custom filter stays `None`.
- Undo: `delete_columns_keeping_tables` returns each row whose hidden state the delete changed, with its new state,
  so the undo hides again what it showed and shows again what it hid (`Diff::row_hidden_toggled` replaces
  `Diff::row_shown`).
- Eval adapter: the spec's tables are added before the number formats are set, so a `"criteria"` filter lists the
  General text spec2xlsx writes and not the formatted text (engine.rs `add_table`).
- Tests (test_filtered_table_edits.rs): General text against display text with undo (row 2 starts with no row
  record); `blank-rejected` / `blank-kept`; `custom` with `equal`, no operator, and an unevaluated `greaterThan`;
  and `CustomFilter::keeps_text` edge cases.
- numbers.md restates the rule as Excel's re-apply, with the gap list removed.
- Eval delta: probes 6782 → 6768 known failures (the 14 cells of t4895-two-criteria-number-format-delete-one). 0
  regressions in any suite.
