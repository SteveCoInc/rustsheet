---
id: T-4049
title: "A sheet-scoped name added through the API after load is #NAME? to an unqualified reference on its own sheet"
milestone: M8
lane: engine
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A sheet-scoped name defined through the API after the workbook loads resolves (or not) in every cell as Excel's does,
backed by probes that tell the possible rules apart.

## Context
- probes/t4014-sheet-api-name-65 (excel-graph 2026-11-04, T-4014): N scoped to sheet S is added by Graph
  `worksheets('S')/names/add` (65 nested ABS calls), then a FullRebuild. `T!A1 =S!N` is 1, but `S!A1 =N`, typed before
  the name existed, stays `#NAME?`. The workbook-scoped twin (probes/t3264-api-name-65) gives `=N` 1. The engine gives
  S!A1 1 (a known failure of the case).
- Unknown which rule it is: a cell's unqualified name is bound when it is entered and only a workbook-scoped add
  rebinds it; or a sheet-scoped add does not dirty its own sheet's dependants; or a Graph quirk. Probe: the same with
  `S!A1` typed after the add, with a workbook-scoped N present before the sheet-scoped one, and with another cell on S
  edited after the add.
- docs/excel-semantics/syntax.md "Formula nesting depth cap" (the T-4014 paragraph); engine: `insert_defined_name`
  (crates/rustsheet-engine/src/defined_names.rs) and name resolution.

## Acceptance criteria
- [ ] The probes above are filled and their rule is in syntax.md.
- [ ] probes/t4014-sheet-api-name-65 S!A1 passes, or the rule is shown to be a Graph artefact and recorded as such.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 65-call cap itself (T-4014).
