---
id: T-5058
title: "Probe the pivot cell formats a refresh leaves: emptied cells, <formats> under preserveFormatting, cacheField numFmtId, a field format on date groups"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's number formats for the pivot cells T-4717 could not observe are evidenced and recorded in pivots.md ("A
refresh gives pivot cells their fields' number formats"), each finding implemented or owned.

## Context
T-4717 (probes/t4717-pivot-cell-formats) shows that a refresh gives each number label its pivot field's `numFmtId` or
General, and each value cell its data field's `numFmtId` or General, whatever format the cell had. Its probe reads the
displayed text only (`Range.text`; CELL("format") is `#VALUE!` on Excel for the web, probes/t3011-cell-format-info-types),
so these readings stay the engine's own (UNVERIFIED):
- a cell a refresh empties (the old area past the new one): the engine leaves its format, which no text shows until
  a value lands there. A probe could write a value into such a cell after the refresh (a Graph range write, as
  probes/t4586-pivot-refresh-on-load C does) and read its text;
- a caption or text label cell (`Row Labels`, `Grand Total`, a text item) over a format with a text section
  (`;;;`, `"x"@`): the engine leaves its format;
- the pivot's `<formats>` (pivotArea formats whose dxf carries a numFmt), which `preserveFormatting` is about: the
  engine keeps them opaque and applies none, so a value cell they format shows its data field's format;
- a non-zero `cacheField@numFmtId` in the file (the builder writes 0, and a `refreshOnLoad` cache is rebuilt), on a
  pivot without `refreshOnLoad` and after a refresh (catalog PV-0362);
- a pivot field `numFmtId` on a grouped field (a date group or a numeric range group): the engine shows the group
  items in the field's format, before the date group item's m/d/yyyy (T-4957);
- a pivot field `numFmtId="0"` on an ungrouped date field: the engine shows General, not the fixed short date of
  T-4807;
- a custom pivot field `numFmtId` (164 and up; numbering is T-4730's) on an item label and a subtotal label;
- a caption filter over a number item of a field with a `numFmtId`: the engine compares its General text
  (`item_text`), not the formatted label.
Code: `item_format`, `item_text_in` and `Cells::value` in crates/rustsheet-engine/src/pivot/layout/render.rs, and
`Model::set_pivot_number_format` in crates/rustsheet-engine/src/pivot/grid.rs.

## Acceptance criteria
- [ ] A probe filled with excel-graph covers the readings above, or pivots.md says why one cannot be observed.
- [ ] The probe shows what an ungrouped date field with `numFmtId="0"` on its pivotField shows (General, as the engine
  does, or the fixed short date), and the engine matches or the cells have an owner.
- [ ] Every visibly-off cell has an owner task (new-case owners gate).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot styles, fonts, fills and conditional formats.
- Date item labels with no field format (T-5049).
