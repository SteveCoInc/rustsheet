---
id: T-4276
title: "Keep an unused shared item and showAll of an ungrouped pivot field (GETPIVOTDATA 0)"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
GETPIVOTDATA on a recordless item of an ungrouped pivot row field answers what Excel does: `#REF!` with `showAll="0"`, 0 with `showAll="1"` (case-insensitive item match).

## Context
Follow-up of T-4248. Owner of probes/t4248-getpivotdata-recordless-item cells P!J1 and P!J5 (Excel 0, engine `#REF!`): the engine builds a pivot's cache from its records and drops a `<sharedItems>` entry no record uses, and ignores `showAll`. Rule: docs/excel-semantics/web.md "GETPIVOTDATA on a recordless item of an ungrouped pivot field". Code: crates/rustsheet-xlsx/src/import/pivots.rs, crates/rustsheet-engine/src/pivot.rs, functions/lookup_and_reference/pivot_index.rs (`shown_group`). The eval adapter ignores the spec's `unused_items` and `show_all`, so it needs them too (crates/rustsheet-eval/src/spec_pivots.rs).

## Acceptance criteria
- [ ] Cells P!J1 and P!J5 of probes/t4248-getpivotdata-recordless-item pass; H1 and H5 (showAll 0) keep passing.
- [ ] The importer keeps an unused shared item and `showAll`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unused items of a grouped field (the probe covers only an ungrouped row field).
