---
id: T-4664
title: "Probe what a pivot refresh does around other pivots, shared caches and gone fields"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md states, with filled probes, what Excel for the web does on a pivot refresh in the cases the grid writer (T-4151) decides without evidence.

## Context
T-4151's grid writer (crates/rustsheet-engine/src/pivot/grid.rs) follows probes/t4128-pivot-spike S for a foreign constant inside the new layout, and takes the engine's own reading for the rest (pivots.md "The grid writer's own readings", UNVERIFIED):
- (a) a layout that would meet another pivot's area, including where that pivot shows a blank intersection: `#SPILL!` at the anchor, as for a constant;
- (b) a foreign constant in the page area above the location (page fields), not in the body: `#SPILL!` too;
- (c) two pivots over one cache: refreshing one refreshes the other (both `refreshOnLoad`, a source formula that changes the values, and only one pivot refreshed through the oracle's refresh call if Graph offers one);
- (d) a refresh after a source header that a row or data field uses is renamed or removed: the engine refuses the refresh and keeps the old cells; Excel is believed to drop the field from the pivot;
- (e) a formatted but empty cell in the layout: not foreign;
- (f) a constant inside the pivot's own saved area (location) that the refreshed layout no longer covers: cleared (T-2327 asks the overwrite question for cells the layout still covers).

## Acceptance criteria
- [ ] Filled probes for (a) to (f); a pivots.md entry for each, replacing the UNVERIFIED readings, and an engine follow-up task where Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto Refresh (T-4182) and refresh without `refreshOnLoad` (T-4586).
