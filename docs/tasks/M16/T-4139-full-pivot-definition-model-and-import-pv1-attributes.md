---
id: T-4139
title: "Full pivot definition model and import (PV1 attributes typed, the rest verbatim)"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4121]
---

## Goal
The workbook model holds every pivot definition attribute that PV1 needs, typed (layout flags per pivot and per field, subtotal flags and placement, grand totals, items with h/sd/s, page field selections, data field names/aggregates/number formats, captions), and keeps every other element and attribute verbatim for round trip; the importer fills it under the SPEC-0015 limits.

## Context
SPEC-0032; crates/rustsheet-engine/src/pivot.rs; crates/rustsheet-xlsx/src/import/pivots.rs. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Unit tests importing the spike and corpus shapes.
- [x] No GETPIVOTDATA regression.
- [x] `make check` green; reviewers approve.

## Out of scope
- Rendering.

## Outcome
The definition model is `crates/rustsheet-engine/src/pivot/model.rs` (moved out of pivot.rs with the existing types):
- **Per pivot:** `PivotTable` gains `page_fields`, `options` (`PivotOptions`) and `opaque`, and a `PivotTable::new`
  constructor that every builder now uses. `PivotOptions` types compact, outline, outlineData, compactData, both
  grand totals, dataOnRows, dataPosition, the four captions, showMissing/missingCaption, showError/errorCaption,
  pageWrap and pageOverThenDown, with the schema's defaults.
- **Per field:** `PivotField` gains compact, outline, subtotalTop, defaultSubtotal, the custom subtotal set (as the
  `ItemType` of each subtotal line), sortType, insertBlankRow, numFmtId and `items` (`PivotItem`: `t`, `x`, `sd` as
  expanded, `s` as a manual filter, per MS-OI29500). `h` of a cache item stays in the hidden bitset GETPIVOTDATA reads.
- **Data and page fields:** `DataField` gains showDataAs, baseField/baseItem and numFmtId (`Default` is the
  schema's); `PageField` holds `fld`, `item` and `hier`.
- **Verbatim:** every other attribute and element is an `OpaqueEntry` (owner element, attributes or a whole child
  element as written); the collections' `count` is derived. The importer (`crates/rustsheet-xlsx/src/import/pivots/table.rs`,
  over a `walk_keeping` that reads unread subtrees verbatim) charges each entry to a workbook budget the size of
  `max_uncompressed_bytes`, and caps a field's items at its cache items plus 14; both are in web.md's DEVIATION entry.
- **Tests:** table/tests.rs imports the t4137 K1/K3/K4/K5, t4128 CalcTopSort, t4138 PG6/PG8 and t4123 Tabular
  definitions (fixtures built by `rustsheet-spec2xlsx`) and `tests/example.xlsx`'s Excel-saved pivot, and checks that
  every attribute of each is typed or kept.
- **GETPIVOTDATA:** unchanged. A data field with showDataAs other than normal is now typed but still skips its pivot
  until T-4161/T-4163; filters and calculated items stay verbatim for T-4165/T-4175. `make eval` shows no regression.

Open: which function `countSubtotal`/`countASubtotal` compute (K5 labels a `count` line `Count` over numbers only),
left to the renderer task's probe.
