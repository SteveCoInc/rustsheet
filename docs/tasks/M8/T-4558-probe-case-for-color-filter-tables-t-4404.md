---
id: T-4558
title: "Probe case for color-filter tables (T-4404): spec2xlsx writes a colorFilter, with excel-graph oracle sidecar, and a dangling dxfId in memory vs export"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine's SUBTOTAL for color-filter tables is checked against Excel by a committed probe case with an excel-graph oracle sidecar, and the in-memory state of an imported dangling `dxfId` agrees with what the export writes.

## Context
- From the T-4404 excel-fidelity review (PR #3568). T-4404 recorded the excel-graph values in xlsx-storage.md ("Table parts in the export") and fixed `has_active_criterion` (crates/rustsheet-xlsx/src/import/tables.rs) with a unit test, but committed no probe case: spec2xlsx (tools/src/rustsheet_tools/spec_tables.py) cannot write a `<colorFilter>`.
- Cases to add, modelled on probes/t4003-filter-column-criterion: a `<colorFilter/>`, a `<colorFilter cellColor="0"/>` and a control, `Sales` A1:B7, rows 3 and 5 hidden, D1:D4 = SUBTOTAL 9, 109, 3, 103; expected 21, 15, 6, 4.
- Open: a workbook with a dangling `dxfId` imports `has_filters = true`, but the export drops the id and writes `<colorFilter/>`, which Excel reads as manual hides, so SUBTOTAL changes across a save. Excel refuses the source file itself.

## Acceptance criteria
- [x] spec2xlsx can write a color filter; the probe case and its oracle sidecar are committed and every cell passes.
- [x] A dangling `dxfId` leaves in-memory and exported SUBTOTAL equal; xlsx-storage.md says which way.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- spec2xlsx writes a colour filter (`filter_column: {"color": true, "cell_color": false}`); the eval case schema and
  adapter read it as criterion-less.
- probes/t4558-color-filter-subtotal (+ excel-graph sidecar): Color, FontColor and Manual all read 6, 4, 21, 15; 12/12 pass.
- Import: a colour filter whose `dxfId` is past the workbook's dxfs leaves the table unfiltered, equal to the export
  (unit test `dangling_color_filter_dxf_is_no_criterion`); xlsx-storage.md says which way.
