---
id: T-4969
title: "A page field with hidden items but no multipleItemSelectionAllowed shows (All)"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A page field with a hidden item but no `multipleItemSelectionAllowed` shows the caption Excel saves for it (`(All)`), not `(Multiple Items)`.

## Context
Found by T-4906, the pivot-corpus render score. pivot-corpus-lo's `xlsx/pivottable_rowcolpage_field_filter.xlsx` Sheet2 pivot has a page field (`fld="5"`, no selected item) whose second item is `h="1"`, and no `multipleItemSelectionAllowed`: Excel's saved B2 reads `(All)`, the engine's `(Multiple Items)`. The pivot's values agree with the engine's record filter (it is fresh), so only the caption differs. The file was first written by LibreOffice, so the caption may be its render; a probe decides.

The rule and its evidence: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)". Related: T-4921 (one visible item shows that item), T-4657. Code: crates/rustsheet-engine/src/pivot/layout/render.rs (`page_area`). The engine pivot code is another stream's: this task is that stream's to implement. The rule is UNVERIFIED (Excel-saved corpus cells, not an oracle probe): probe it first with the excel-probe skill (pivots.md's evidence rule).

Split: a corpus pivot can show several causes; each T-4906 owner task (T-4966 to T-4970) and the gen-pivot owners it names own their own cause there.

Owner of these visibly-off cells (decision 0245), 1 render cell of 1 fresh pivot:
- pivot-corpus-lo/xlsx/pivottable_rowcolpage_field_filter.xlsx Sheet2!B2

## Acceptance criteria
- [ ] A probe case shows the page caption and the values Excel for the web renders for hidden page items with and without `multipleItemSelectionAllowed`, and pivots.md's entry cites it.
- [ ] The engine's page caption (and its record filter, if the probe shows the flag changes it) follows.
- [ ] The listed pivot renders exactly.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A page field with one visible item (T-4921).
