---
id: T-4764
title: "Probe pivot repeat item labels beyond the T-4698 shapes"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4698]
---

## Goal
The engine's readings of Repeat Item Labels (`x14:pivotField@fillDownLabels`) beyond the shapes probes/t4698-pivot-repeat-labels shows are evidenced, and corrected where Excel differs.

## Context
Split from T-4698, which implemented the probed shapes (crates/rustsheet-engine/src/pivot/layout/render.rs `Cells::fill_down`; pivots.md "Repeat item labels"). Engine readings left UNVERIFIED (pivots.md, T-4764):
- a flagged level whose label column it shares with a deeper level the line does not label (a compact, flagged outer field over an unflagged tabular field that has a field inside it): the engine fills nothing, the cell going to the deeper level (t4698 MX evidences only the flagged deeper level);
- on columns with three fields, an inner field's subtotal column: the engine repeats the flagged outer label above it, as rows do (t4698 T3R!A6; CC has two column fields only);
- the Values field on the same axis as a flagged field (several data fields on rows or columns), and a Values level between flagged fields: the engine fills field levels only;
- an explicit `fillDownLabels="0"` beside `fillDownLabelsDefault="1"`, and custom subtotal lines (`sumSubtotal`...) under a flagged outer field (the engine fills them as it fills default subtotal lines).
- whether `fillDownLabelsDefault="1"` is the default for fields added later (pivots.md "Layout and totals flags", X14 bullet): a file cannot add a field, so record what a probe can show (if nothing, say so and keep it UNVERIFIED with the reason).
Write `probes/t4764-...` with the case builder's per-field `xml.extLst` slot and fill it with the excel-graph oracle.

## Acceptance criteria
- [ ] A filled probe case covers each reading above; pivots.md records the result with the evidence cells.
- [ ] The engine's rendered cells pass the probe in unit tests against the sidecar (crates/rustsheet-engine/src/pivot/layout/render/tests.rs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Styles and indentation.
