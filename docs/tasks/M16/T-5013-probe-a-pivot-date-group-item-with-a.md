---
id: T-5013
title: "Probe a pivot date group item with a time part, on columns, as a page item and in its subtotal label"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4957]
---

## Goal
pivots.md "Capability matrix 3" says, from Excel for the web's evidence, how a pivot date group item
(`fieldGroup/groupItems/d`) shows wherever T-4957's m/d/yyyy reading is only the engine's own.

## Context
T-4957 shows a date group item's label cell as its serial in m/d/yyyy (built-in id 14), from
probes/t4767-pivot-group-items GD!A6 and GDA!A6: one midnight date (`<d v="2020-01-01T00:00:00"/>`), a row label of a
manual group over a text field. The engine's readings no probe shows
(crates/rustsheet-engine/src/pivot/layout/render.rs `item_format`; pivots.md "Capability matrix 3", UNVERIFIED): a `d`
item with a time part (m/d/yyyy still, so the time is not shown); the item as a column label and through Repeat Item
Labels (the same format); the item as a page field's selection (its serial, no format); its subtotal label (`43831
Total`, `number_to_text` of the serial); a `d` item in a range (rangePr) group's groupItems (m/d/yyyy); and a manual
group over a date field, whose ungrouped base items are date items of the group field (m/d/yyyy too, not the source
cells' format).

## Acceptance criteria
- [ ] An excel-probe case covers each reading above; pivots.md records what Excel shows, and a follow-up engine task is
      filed for each reading the engine does not match.
- [ ] The probe records the label cell's `numFmtId` (the engine writes built-in 14 for m/d/yyyy; T-4957).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date items of an ungrouped date field (T-4807).
