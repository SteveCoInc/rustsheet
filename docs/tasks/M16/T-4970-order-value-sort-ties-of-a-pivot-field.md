---
id: T-4970
title: "Order value-sort ties of a pivot field sorted by a show-values-as data field as Excel does"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Items of a pivot field value-sorted by a show-values-as data field whose sort values tie come out in Excel's order.

## Context
Found by T-4906, the pivot-corpus render score. pivot-corpus-lo's `xlsx/column-style-autofilter.xlsx` TopDesktopDistrosProfit (TopDesktopDistrosVsServerUsage!A3:C64) sorts its row field `sortType="descending"` with an `autoSortScope` on its one data field, a Count shown as `percentOfRow` over a column field, with `colGrandTotals="0"`. From row 19 (Ubuntu GNOME, Antergos, both with 19 records) Excel's order differs from the engine's cache-order tie break (pivots.md "Value sorts: ties, blanks and errors"), so 76 cells of 15 rows differ. Which value Excel sorts by under a show-values-as mode (the shown percentage, all 1 at the row total, or the plain count) and how it breaks ties there is unprobed.

The rule and its evidence: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)". Related: T-4799 (ties of blanks, zeros and errors). Code: crates/rustsheet-engine/src/pivot/sort.rs (`value_sorts`). The engine pivot code is another stream's: this task is that stream's to implement. The rule is UNVERIFIED (Excel-saved corpus cells, not an oracle probe): probe it first with the excel-probe skill (pivots.md's evidence rule).

Split: a corpus pivot can show several causes; each T-4906 owner task (T-4966 to T-4970) and the gen-pivot owners it names own their own cause there.

Owner of these visibly-off cells (decision 0245), 76 render cells of 1 fresh pivot:
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx TopDesktopDistrosVsServerUsage!A19:C63

## Acceptance criteria
- [ ] A probe case shows the order Excel for the web gives a value sort by a show-values-as data field, with ties, and pivots.md's entry cites it.
- [ ] The engine's value sort follows, and the listed pivot renders exactly.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Value sorts without a show-values-as mode (T-4827, done; T-4799).
