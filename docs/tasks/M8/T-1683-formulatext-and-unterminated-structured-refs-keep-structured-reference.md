---
id: T-1683
title: "FORMULATEXT and unterminated structured refs: keep structured-reference text; reject t[@Qty without closing bracket"
milestone: M8
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT of a formula with a structured reference shows what Excel shows, and an unterminated structured reference (`=T[@Qty`, `=T[Col`) is a parse error.

## Context
- Since T-2195 every structured reference except `#This Row` is kept as written, and FORMULATEXT shows the canonical long form the storage printer writes (`=SUM(Sales[Qty])`, the table name in its typed case, `Sales[]` as bare `Sales`); `#This Row` is still lowered, so FORMULATEXT shows `=$A$2` for `=T[@Qty]` until T-1699. The probe should also cover the case of column names as typed (`sales[qty]`), whether Excel drops a redundant `[#Data]`, and the `T[[Col]]` → `T[Col]` spelling. Excel's FORMULATEXT output is unprobed: use the `excel-probe` skill first.
- `consume_escaped_until` (lexer/structured_references.rs) accepts input with no closing `]` and moves the position past the end; found in T-1672 review. Confirm Excel rejects it by probe.

- Evidence for the `#This Row` spelling (excel-graph, 2026-10-28, T-2845; references.md "Structured references when
  the table changes"): FORMULATEXT prints `=Sales[@Qty]` for a stored `Sales[[#This Row],[Qty]]`, and `=[@Qty]*2` for
  `=Sales[@Qty]*2` inside table Sales. Ours prints the long form with the name. Cases: probes/table-edits
  table-insert-row-this-row-above-header, -this-row-inside-data, -calc-column-above-header, -calc-column-inside-data
  (the FORMULATEXT cells; their #This Row values are T-3037's).
- Gap triage 2026-11-01 (`make eval` on origin/main fafb6c6f): `scripts/exit_owners.py --milestone M8` read the 11
  FORMULATEXT cells of the inside-data case as M8's only unowned exit cells, because the list above abbreviates
  its name. Owned cells: probes/table-edits/table-insert-row-this-row-inside-data.case.json F!B4:B8, F!G1:G6
  (Excel "=Sales[@Qty]", ours "=Sales[[#This Row],[Qty]]").

## Acceptance criteria
- [x] Probe Excel FORMULATEXT for `T[@Qty]`, `T[[#This Row],[Qty]]`, `T[Qty]`; record in docs/excel-semantics/references.md.
- [x] Include a bare table name (`ROWS(Sales)`), now lowered to a range like `Sales[]`.
- [x] Implement the probed behavior; unterminated references give a lexer error.
- [x] Probe whether Excel rejects the unbracketed multi-word `T[@Unit Price]` (the lexer accepts it now); reject if so.
- [x] `make check` green; reviewers approve.

## Out of scope
- Rows outside the table (T-1675).

## Outcome
- Probe `probes/t1683-formulatext-structured-refs` (61 cells, excel-graph 2026-11-04). Excel prints a `#This Row`
  reference in its `@` form with the table name (`=Sales[[#This Row],[Qty]]` → `=Sales[@Qty]`, `Sales[#This Row]`
  → `Sales[@]`, `Sales[@[Unit Price]]`). It also prints table and column names in the table's own spelling
  (`=SUM(sales[qty])` → `=SUM(Sales[Qty])`, `=ROWS(sales)` → `=ROWS(Sales)`). It refuses every typed
  unterminated reference, which the lexer already did. It accepts the unbracketed multi-word `=Sales[@Unit Price]`
  and prints it `=Sales[@[Unit Price]]`, so that form stays accepted. Recorded in references.md
  "Structured references: FORMULATEXT"; SPEC-0016 "Printing" updated.
- Engine: `StructuredReference::text_with_name` prints the `@` form with or without the name (`prints_bare_after_at`
  brackets a name with whitespace). `StructuredReference::use_table_spelling` (called from
  `structured_reference_operand`) takes the table's spelling of the names. The xlsx export still writes the long
  form (`file_text`). Tests: the new probe's cells, `multi_word_this_row_column_and_unterminated_references`, and
  `every_written_form_is_kept` rows. The expectations of 12 existing tests change from the long form to the `@` form.
- Eval delta: probes 4124 → 4102 known failures (22 fixed: the FORMULATEXT cells of
  table-insert-row-this-row-inside-data and -this-row-above-header F!B4:B8, F!G1:G6), +1 case (61 cells, all
  pass), 0 regressions.
