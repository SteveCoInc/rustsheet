---
id: T-4967
title: "Leave a pivot value cell blank, not an empty string, under an empty missingCaption"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Where a pivot's `missingCaption` is empty, the cells no record reaches are left blank, not written as an empty string, as Excel saves them.

## Context
Found by T-4906, the pivot-corpus render score. Every pivot of pivot-corpus-closedxml's `PivotMixedDataTypesInTableColumn/input.xlsx` (and `PivotTableWithNoneTheme/inputfile.xlsx` pvt2) has `missingCaption=""` with `showMissing` on: Excel's saved empty intersections (a `percentOfItem` grand total, a cell no record reaches) hold nothing; the engine writes the text `""`. The freshness test already reads an empty caption as a blank (crates/rustsheet-eval/src/pivot_freshness/records.rs).

The rule and its evidence: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)". Related: T-4922 (a `missingCaption` of `-`). Code: crates/rustsheet-engine/src/pivot/layout/render.rs (the missing caption). The engine pivot code is another stream's: this task is that stream's to implement. The rule is UNVERIFIED (Excel-saved corpus cells, not an oracle probe): probe it first with the excel-probe skill (pivots.md's evidence rule).

Split: a corpus pivot can show several causes; each T-4906 owner task (T-4966 to T-4970) and the gen-pivot owners it names own their own cause there.

Owner of these visibly-off cells (decision 0245), 52 render cells of 6 fresh pivots:
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotMixedDataTypesInTableColumn/input.xlsx pvt1!G3:G4
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotMixedDataTypesInTableColumn/input.xlsx pvt2!B8:C8
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotMixedDataTypesInTableColumn/input.xlsx pvt3!B12
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotMixedDataTypesInTableColumn/input.xlsx pvt4!G4
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotMixedDataTypesInTableColumn/input.xlsx pvt5!B3:G19
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotTableWithNoneTheme/inputfile.xlsx pvt!B8:C8

## Acceptance criteria
- [ ] A probe case shows an empty `missingCaption` on an empty intersection and on a show-values-as total with no base, and pivots.md's entry cites it.
- [ ] The engine leaves those cells empty (no stored cell), and GETPIVOTDATA on them answers as the probe shows.
- [ ] The listed pivots render exactly.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other missing captions (T-4922).
