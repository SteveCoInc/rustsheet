---
id: T-4711
title: "Probe: Excel reads a shared-string index with surrounding whitespace (poi/bug69769 B2)"
milestone: M11
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's reading of a `t="s"` cell whose `<v>` index has surrounding whitespace is evidenced on excel-graph, and the semantics entry loses its UNVERIFIED status.

## Context
T-4145 made the importer read `<v>2 </v>` as index 2 (poi/bug69769.xlsx Tabelle1!B2, written by Excel Online 16.0300) from the file alone; no oracle value shows what Excel does. docs/excel-semantics/xlsx-storage.md "Shared-string index with surrounding whitespace" is UNVERIFIED. Probe (excel-probe skill): a workbook with `<v>2 </v>`, `<v> 2</v>`, `<v>+2</v>` `<v>-1</v>` and a non-breaking-space padded index plus `=B2`/`=LEN(B2)` cells; read the values. Code: `parse_shared_string_index` in crates/rustsheet-xlsx/src/util.rs.

## Acceptance criteria
- [ ] Probe case recorded under probes/ with oracle values for the four shapes.
- [ ] The semantics entry cites it, and `parse_shared_string_index` follows it (trim set, leading `+`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
