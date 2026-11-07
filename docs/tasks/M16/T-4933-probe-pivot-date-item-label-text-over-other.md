---
id: T-4933
title: "Probe pivot date-item label text over other source formats, time-only items and seconds"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine shows an ungrouped pivot date item's label as Excel does for every source format and item shape,
decided by a probe with pivot `text`, and pivots.md says exactly what the engine does where it is unprobed.

## Context
Follow-up of T-4807's Excel-fidelity review (PR #3952, three MINOR findings). T-4807 gives a `CacheItem::Date`
label cell `m/d/yyyy`, or `m/d/yyyy h:mm` when the serial has a fractional part (`date_item_format`,
crates/rustsheet-engine/src/pivot/layout/render.rs), from t2085-getpivotdata-items D, t4130-pivot-date-filters
(source `m/d/yyyy`) and t4164-pivot-date-times (source `m/d/yyyy h:mm`). Unprobed:
- an ungrouped date row field over another source format (`yyyy-mm-dd`, `d-mmm-yy`, `dddd`): is the label still
  `m/d/yyyy`? (t4170's group boundaries `<1/1/2019` over a `yyyy-mm-dd` source suggest so);
- a time-only item below 1 from an `h:mm` or `[h]:mm` source (refresh.rs reads these as dates, t4612): the engine now
  shows `1/0/1900 12:00` for 0.5. pivots.md "Item casing when cache and source disagree, and date items" wrongly says
  such an item "keeps its earlier rendering"; correct it in this task whatever the probe finds;
- a time part with seconds (45000.5000115) and a serial a hair off an integer (the `fract() == 0.0` cut);
- a date item's subtotal label (`<item> Total`) and a selected date page item.
probes/t4612-pivot-date-items-and-casing has `h:mm` sources but no `text` in its sidecar; a new case (excel-probe,
excel-graph, the sidecar's pivot `text`) is simpler than a refill.

## Acceptance criteria
- [ ] A probe case filled on excel-graph covers the shapes above; pivots.md records what Excel shows.
- [ ] The engine matches every probed label cell (or a named owner task holds the rest).
- [ ] pivots.md no longer says a time-only item keeps its earlier rendering unless the engine does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Grouped date fields (their labels are text items).
