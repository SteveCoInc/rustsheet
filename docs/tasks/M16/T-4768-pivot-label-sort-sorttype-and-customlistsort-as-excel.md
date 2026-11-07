---
id: T-4768
title: "Pivot label sort: sortType and customListSort as Excel for the web renders them"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4130]
---

## Goal
A pivot field's label sort renders in the order Excel for the web renders it: `sortType` ascending and descending
over the field's items, `manual` in the field's listed order, and with `customListSort` on (the default) the built-in
custom lists ahead of the alphabetical order (the day names are evidenced; the month lists are Excel's other
built-in lists, unprobed).

## Context
- T-4130 (probes/t4130-pivot-sort; pivots.md "Capability matrix 2" and "Pivot sorts and filters as Excel for the web
  renders them"): the engine renders every field in its cache order whatever `sortType` says. Excel sorts SA
  ascending (Ant, Bee, Cat), SD and NA descending (Cat, Bee, Ant; `nonAutoSortDefault` changes nothing visible), CL1
  Day ascending by the day-name custom list (Mon, Wed, Fri) and CL0, with `customListSort="0"`, alphabetically (Fri,
  Mon, Wed). SM (manual) already passes.
- Value sorts (`autoSortScope`) are T-4165's. Code: crates/rustsheet-engine/src/pivot/result.rs (`Order`).
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245):
- probes/t4130-pivot-sort SA!A4:A5, SA!B4:B5, SD!A4:A6, SD!B4:B6, NA!A4:A6, NA!B4:B6, CL1!A4:A5, CL1!B4:B5, CL0!A4, CL0!A6, CL0!B4, CL0!B6

## Acceptance criteria
- [ ] The engine renders SA, SD, NA, CL1 and CL0 as the sidecar shows (unit tests against it, and the cells pass in the eval).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sorting by value (T-4165) and user-defined custom lists (not in the file).
