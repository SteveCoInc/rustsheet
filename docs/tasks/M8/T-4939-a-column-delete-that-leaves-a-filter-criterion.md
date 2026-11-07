---
id: T-4939
title: "A column delete that leaves a filter criterion re-applies the kept criteria to every data row: hide visible rows they reject, evaluate custom filters"
milestone: M8
lane: engine
status: in-progress
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
- [ ] probes/t4895-two-criteria-number-format-delete-one passes (its baseline failure line goes).
- [ ] First a positive control, probed with predictions committed before the fill: the same `0.00` column with its kept
      filter listing the displayed text ("10.00" .. "60.00"), deleting Cat. The display-text rule rests on one probe
      where nothing matched, so this control must show the rows the filter keeps before the engine implements it.
- [ ] Unit tests, in the engine's table-edit tests, for the `blank-rejected` and `custom` uploads (rebuild them with
      docs/research/t4895/uploads.py) and for a rejected visible row with no stored row record.
- [ ] numbers.md's rule is restated as Excel's re-apply, with the gaps removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Kept top10, dynamic, date-group and color criteria (unprobed; they stay as they are).
- Re-applying filters after a value edit.
