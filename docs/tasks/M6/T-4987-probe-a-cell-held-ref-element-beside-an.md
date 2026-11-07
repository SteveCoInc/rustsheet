---
id: T-4987
title: "Probe a cell-held #REF! element beside an off-sheet reference in LARGE/SMALL/MEDIAN (T-4862 skips every #REF! by kind)"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
LARGE/SMALL/MEDIAN over a grid holding a cell-held `#REF!` element beside an off-sheet reference give what Excel gives, evidenced by oracle cells.

## Context
T-4862 made `RefGrid::order_statistic_error` (crates/rustsheet-engine/src/reference_operators/areas.rs) skip every `#REF!` element by kind, evidenced only for a failed INDIRECT (probes/t4791 H4:H6). A `#REF!` a cell holds (XLOOKUP hit on a `#REF!` cell, in both orders; OFFSET/INDEX out-of-range elements if they occur in such grids) is unevidenced. Notes: lookup.md "A grid with references on both sheets".

## Acceptance criteria
- [ ] A probe case with oracle values for the cell-held `#REF!` grids; the code follows by kind or by origin as the oracle shows; lookup.md and the code comment agree.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
