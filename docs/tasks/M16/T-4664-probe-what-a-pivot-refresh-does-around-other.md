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
- (a) a layout that would meet another pivot's area, including where that pivot shows a blank intersection: the engine writes `#SPILL!` at the anchor, as for a constant. The competing reading is classic Excel's, which refuses a refresh that would overlap another pivot with a dialog (excel-docs.md "Overlap rules"), so the probe must tell `#SPILL!` from a refused refresh (the old cells and `location` kept);
- (b) a foreign constant in the page area above the location (page fields), not in the body: `#SPILL!` too;
- (c) two pivots over one cache: refreshing one refreshes the other (both `refreshOnLoad`, a source formula that changes the values, and only one pivot refreshed through the oracle's refresh call if Graph offers one);
- (d) a refresh after a source header that a row or data field uses is renamed or removed: the engine refuses the refresh and keeps the old cells; Excel is believed to drop the field from the pivot. Also a data field shown as `% of` a base field whose header is removed: the engine falls back to the first field (`baseField="0"`);
- (e) a formatted but empty cell in the layout: not foreign;
- (f) a constant inside the pivot's own saved area (location) that the refreshed layout no longer covers: cleared (T-2327 asks the overwrite question for cells the layout still covers).
- (g) a merged range in the layout, with and without text in it: the engine collapses to `#SPILL!` (PSPILL in excel-docs.md documents merged cells for desktop Excel);
- (h) an Excel Table whose body is empty inside the layout: `#SPILL!` (PSPILL documents tables);
- (i) a layout that would pass the sheet's last row or column (an anchor a few rows above row 1,048,576): `#SPILL!` (PSPILL documents the sheet edge);
- (j) a collapse of a pivot with page fields: the saved `rowPageCount`/`colPageCount` and whether the page cells above the anchor stay (the engine zeroes the counts and clears the page area).
- (k) a `refreshOnLoad` pivot saved with `mergeItem="1"` ("Merge and center cells with labels") and merged label cells, then refreshed with a source that changes its labels: the engine refuses the refresh and keeps the saved cells, since it cannot write through its own merges;
- (l) an array formula and a dynamic-array spill whose cells fall inside the new layout, and inside the pivot's own saved area: the engine collapses to `#SPILL!` and never clears those cells.

## Acceptance criteria
- [ ] Filled probes for (a) to (l); a pivots.md entry for each, replacing the UNVERIFIED readings, and an engine follow-up task where Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto Refresh (T-4182) and refresh without `refreshOnLoad` (T-4586).
