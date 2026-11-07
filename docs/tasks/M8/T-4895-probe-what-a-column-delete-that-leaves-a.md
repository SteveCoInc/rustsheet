---
id: T-4895
title: "Probe what a column delete that leaves a filter criterion does to hand-hidden, visible, case-differing and formatted rows"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the parts of T-4803's "a column delete that takes one criterion re-applies the kept ones" rule
that are recorded as ours and unprobed (docs/excel-semantics/numbers.md, "SUBTOTAL and AGGREGATE over hidden and
filtered rows"), so the engine follows the evidence there too.

## Context
- T-4803 (PR #3920) shows each hidden data row that every kept criterion keeps, by the row's displayed text
  (`rows_kept_by_criteria`, `FilterCriterion::keeps_text` in crates/rustsheet-engine). Only
  probes/t4755-two-criteria-filtered-table-delete-one backs it: one kept values filter on plain lowercase text, with no
  hand-hidden row.
- Unprobed (review findings on #3920):
  - a row hidden in the file that the kept criteria keep, with two criteria (the engine shows it);
  - a visible row that a kept criterion rejects (the engine leaves it visible; does Excel re-apply the whole filter?);
  - kept values that differ in case from the cells (the engine ignores case);
  - number-formatted values and blank cells in the kept column;
  - a kept custom, date-group, top10 or color criterion (the engine keeps every hidden row hidden).
- Also from the review of #3831 (T-4749): the eval adapter's every-row-hidden rule (`filters_hidden_rows`, which
  counts a values filter with no visible data row as no criterion) reaches `"criteria"` tables too. For those tables
  spec2xlsx writes one `<filters>` per criterion, and that `<filters>` still lists values unless the criterion hides
  every row. No case reaches this today. Once evidence exists, make the adapter count such a table as filtered when
  some criterion keeps a row.
- spec2xlsx's `"criteria"` writes only values filters (tools/src/rustsheet_tools/spec_tables.py). Ad-hoc uploads may
  be needed for the other kinds; record their sha256 values as T-4749 did.

## Acceptance criteria
- [x] A probes case (excel-graph) for each unprobed point above that spec2xlsx can write, with the engine's predictions
      committed before the fill. The other points are probed by upload, with their sha256 recorded.
- [x] numbers.md states each result. Every visibly-off cell has an owner task, or is fixed here if it is size S.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters after a value edit; worksheet autoFilters.

## Outcome
- Finding (excel-graph, 2026-11-07): a column delete that takes one criterion and leaves another makes Excel
  **re-apply the kept criteria to every data row**, matching each row by the text it displays and ignoring case.
  The predictions were committed before each fill and upload (docs/research/t4895/).
- Probe cases (4 cases, 72 cells):
  - `probes/t4895-two-criteria-hand-hidden-{control,delete-one}`: a hand-hidden row that both criteria keep is shown
    by the delete. The engine matches, 36/36.
  - `probes/t4895-two-criteria-number-format-control`: the engine matches, 18/18.
  - `probes/t4895-two-criteria-number-format-delete-one`: a kept values filter listing General text over a `0.00`
    column hides every row, including the visible ones. The engine passes 4/18; the other 14 cells are owned by T-4939.
  - An earlier fill of the hand-hidden delete case, before the `COLUMNS` reader was added, did not reproduce. Its
    sidecars are kept in docs/research/t4895/first-fill/.
- Uploads (5 patched workbooks, sha256 values in docs/research/t4895/predictions-uploads.tsv). The re-apply model
  predicts all 5; the engine's current rule predicts 3.
  - Matching the engine: kept values that differ in case from the cells, and a blank kept cell under `blank="1"`.
  - Differing from the engine (owned by T-4939): a visible blank row with no `blank="1"` is hidden, and a kept custom
    filter shows a row that only the deleted criterion hid.
  - Kept top10, dynamic, date-group and color criteria stay unprobed.
- Not done: the adapter's every-row-hidden rule (`filters_hidden_rows`) for `"criteria"` tables. No probe here has a
  criterion that hides every row, so that rule is still unprobed and the adapter is unchanged.
- numbers.md ("SUBTOTAL and AGGREGATE over hidden and filtered rows") records the rule. Follow-up: T-4939 (engine,
  opus).
- Eval delta: probes +4 cases (72 cells, 58 pass). New known failures: 14 cells of
  t4895-two-criteria-number-format-delete-one, owned by T-4939. Fixed 0, regressions 0.
