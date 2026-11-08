---
id: T-5035
title: "Parse a sheet-qualified corner joined to a function by the range operator (Data!$A$1:INDEX(...)), in cells and defined names"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A formula or defined-name body that joins a sheet-qualified corner to a function by `:`, such as
`Data!$A$1:INDEX(Data!$B:$B,COUNTA(Data!$A:$A))`, parses and evaluates as Excel's range, so a workbook defining
the common dynamic-range idiom loads.

## Context
Owns the whole case probes/t4506-pivot-index-name-corner (an engine error today: the adapter's defined name
is refused as `Invalid formula`). Excel for the web reads `ROWS(IndexSrc)` as 6 (Q!A2) and refreshes a pivot over
the name to five records (I!A3:B8; pivots.md "A pivot over a defined name or a Table reads it again at the
refresh", T-4506). The unqualified `A1:INDEX(...)` and `INDEX(...):INDEX(...)` parse already; the lexer keeps
the sheet-qualified shape a parse error on purpose until probed
(crates/rustsheet-engine/src/expressions/lexer/test/test_ranges.rs
`test_sheet_qualified_valid_corner_pairs_stay_ranges`, `Data!A1:INDEX(Data!B:B,2)`). Probe the cell form too
(`=ROWS(Data!$A$1:INDEX(Data!$B:$B,3))`) before changing the lexer, and check what an xlsx load does with such a
name today (the whole workbook may be refused).

## Acceptance criteria
- [ ] probes/t4506-pivot-index-name-corner passes; a cell-form probe filled and passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other function corners than the probed ones.
