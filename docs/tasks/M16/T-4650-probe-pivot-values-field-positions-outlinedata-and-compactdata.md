---
id: T-4650
title: "Probe pivot Values field positions, outlineData and compactData, and the page-field grid"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md "Layout lines" and "Pivot geometry" settle the Values field away from the innermost position, its `outlineData`/`compactData`, the header offsets that are always 1 in the probes, and the page-field grid's other orders.

## Context
T-4147 emits the lines and location (`crates/rustsheet-engine/src/pivot/layout/lines.rs`). Committed probes put Values innermost only (t4135 `A_sum` columns, t4138 G6 rows). The engine's readings, UNVERIFIED in pivots.md:
- Values outer or in the middle of an axis (`rowFields` Region, -2, Product): the Values level is one more level of the tuple, a field with Values inside gets one subtotal line per data field (`i` the data field) after its children, never on its own line;
- Values on rows with `outlineData="1"`: a line of its own per data field above the inner items; `compactData` and `outlineData` both on: the next level shares its column (`firstDataCol`);
- `firstHeaderRow` is 1 whenever the pivot has a body (mog writes 0 with x14 `hideValuesRow`, oss-engines.md);
- page fields: `rowPageCount`/`colPageCount` for `pageOverThenDown` without `pageWrap`, and for down-then-over with `pageWrap` (only t4137 K4 and t4138 G8 are probed, and G8's 2 by 2 cannot tell the two orders apart);
- the cell text T-4149's renderer (`crates/rustsheet-engine/src/pivot/layout/render.rs`) writes: a subtotal line of a field with Values inside reads `<item> <data field>` (`East Sum of Sales`); a grand-total line on the Values axis reads `Total <data field>` even when `grandTotalCaption` is set (only the unset case is probed: t4138 G6!B11:B12, t4135 `A_sum`!N4:S4); the header cell over a Values level in a label column of its own is left empty.

Case builder v2 with excel-graph.

## Acceptance criteria
- [ ] A filled probe with the variants; each reading above closed or corrected in pivots.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering the cells (T-4149); the page area's capture window (T-4584).
