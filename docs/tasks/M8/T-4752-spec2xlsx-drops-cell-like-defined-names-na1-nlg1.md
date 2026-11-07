---
id: T-4752
title: "spec2xlsx drops cell-like defined names (NA1, NLG1, R0C1, C0) that the engine adapter defines: write them or refuse the spec, and re-check the cases that used them"
milestone: M8
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A case's `names` reach Excel exactly as the engine adapter defines them: a name that XlsxWriter's `define_name` refuses
as cell-like is either written into workbook.xml or refused by spec2xlsx with an error, never silently dropped.

## Context
Found by T-4446. `build_xlsx` (tools/src/rustsheet_tools/spec2xlsx.py, `_define_name`) passes each spec name to
XlsxWriter's `define_name`, which returns -1 with only a warning for a name matching `^[a-zA-Z][a-zA-Z]?[a-dA-D]?\d+$`
(cell-like) or `^[rcRC]$` / `^[rcRC]\d+[rcRC]\d+$` (R1C1-like). The engine adapter (crates/rustsheet-eval/src/engine.rs
`define_names`) still loads them, so oracle and engine see different workbooks. T-4446 made spec2xlsx write names
failing XlsxWriter's character check by hand; the cell-like ones were left as they were to keep existing cases'
`input_sha256`.

Cases whose names are dropped today: probes/legacy-marker-spelling-name-nlg1 (`NLG1`), probes/sheet-ops/t4702-names-delete-row
and -delete-column (`NA1`, whose "probe trap" note in references.md may be this drop rather than a cell read),
probes/r1c1-name-r0c1, -r0c1-lower, -r0, -c0, -r1048577, -r1048577c1, -use-r0c1, -use-r0, -use-c0, -use-r1048577. Re-read each
case's conclusions in docs/excel-semantics/ (references.md, r1c1 notes) once the names reach Excel.

## Acceptance criteria
- [ ] spec2xlsx writes (or refuses with an error) every spec name XlsxWriter refuses; a unit test covers `NA1` and `R0C1`.
- [ ] The cases listed above are refilled on excel-graph (or their names changed), and any conclusion that rested on a
      dropped name is corrected in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
