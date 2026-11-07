---
id: T-4968
title: "Caption a pivot data field saved without a name as Excel does"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot data field saved with no `name` gets the caption Excel shows for it, instead of an empty one.

## Context
Found by T-4906, the pivot-corpus render score. pivot-corpus-lo's `xlsx/pivottable_tabular_mode.xlsx` and `xlsx/pivottable_duplicated_member_filter.xlsx` each have `<dataField fld="3" subtotal="count"/>` with no `name`; Excel's saved header cell reads `mail`, the source field's name alone, where the engine writes `""`. Both pivots were first written by LibreOffice, so the saved caption may be LibreOffice's render that Excel kept (pivots.md "Pivots without refreshOnLoad keep their saved cells"); a probe decides whether Excel writes `mail` or `Count of mail`.

The rule and its evidence: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)". Code: crates/rustsheet-xlsx/src/import/pivots/table.rs (`DataField::caption`), crates/rustsheet-engine/src/pivot/layout/render.rs. The engine pivot code is another stream's: this task is that stream's to implement. The rule is UNVERIFIED (Excel-saved corpus cells, not an oracle probe): probe it first with the excel-probe skill (pivots.md's evidence rule).

Split: a corpus pivot can show several causes; each T-4906 owner task (T-4966 to T-4970) and the gen-pivot owners it names own their own cause there.

Owner of these visibly-off cells (decision 0245), 2 render cells of 2 fresh pivots:
- pivot-corpus-lo/xlsx/pivottable_tabular_mode.xlsx pivot!C3
- pivot-corpus-lo/xlsx/pivottable_duplicated_member_filter.xlsx pivot!B3

## Acceptance criteria
- [ ] A probe case shows the caption Excel for the web renders (and saves as `name`) for a data field written without `name`, for Sum and Count, and pivots.md's entry cites it.
- [ ] The importer or renderer gives that caption, and GETPIVOTDATA's data-field argument matches it as Excel's does.
- [ ] The listed pivots render exactly, or their cells are shown to be another program's render and the entry says so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Captions of named data fields.
