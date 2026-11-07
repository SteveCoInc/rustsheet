---
id: T-4129
title: "Oracle capability matrix 1: layout, totals, aggregates and show-values-as rows"
milestone: M16
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4122, T-4125]
---

## Goal
For every catalog row in the layout, totals, aggregation and show-values-as families, a minimal probe records whether Excel for the web opens, refuses, repairs or renders it, in the catalog's `web` column.

## Context
Graph cannot create pivots; Excel for the web renders authored definitions (oracle-spike.md). The Excel 2010+ show-as modes live in x14 extensions: whether the web honours them is the first question (excel-docs.md). Rows the web cannot render go to M14 (decision 0039) and never block M16. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] `web` filled for every row of these families, each citing a probe.
- [x] Refused rows have M14 tasks: none filed, because decision 0266 closes M14 to new tasks; the refusal is recorded in pivots.md instead.
- [x] `make check` green; reviewers approve.

## Out of scope
- Edge-pool coverage (later probe tasks).

## Outcome
All 143 catalog rows of the layout, totals, aggregation and show-values-as families have `web` filled, and each
cites a probe and sheet in `evidence`: 140 are `yes` and 3 are `no`.
- **Render (yes).** These rows render:
  - every ST_DataConsolidateFunction, ST_ItemType and ST_ShowDataAs value;
  - all six x14 ST_PivotShowAs modes; the cached downloads (not a committed file) show Excel saves them as written, as the x14 dataField `pivotShowAs` with no
    `showDataAs`;
  - every location, axis and totals attribute;
  - the x14 pivotTableDefinition and pivotField extensions.
- **Refuse (no).** Three flags make the web refuse the file (`FileCorruptTryRepair`): `pivotField@dataSourceSort`,
  `item@d` and `item@f` (with no calculatedItems). No M14 task is filed:
  decision 0266 closes M14 to new tasks, so the refusal is recorded in pivots.md, and decision 0274 makes these
  rows non-blocking.
- **Repair.** Nothing was repaired on open. Excel silently drops `item@m="1"`, `item@e="1"` (the default) and a
  pivotField's `axis="axisValues"` when it saves.

New probes, all excel-graph, 2026-11-06:
- probes/t4129-pivot-show-as (the nine ECMA modes);
- probes/t4129-pivot-show-as-x14 (the six x14 modes);
- probes/t4129-pivot-layout-flags (12 layout and totals flags);
- probes/t4129-pivot-item-flags (items with `c`, `e` or `m`).

Their findings are in pivots.md under four new entries (capability matrix, refused flags, show-as values, layout
flags). The show-as sidecars are evidence for T-4161 and T-4163. Among the findings:
- `#NULL!` for an empty intersection under % Of and % Difference From;
- `subtotalCaption` replaces the whole label;
- with `subtotalHiddenItems` set, a hidden row item is left out of its subtotal and the grand total (flag-off control: T-4164);
- `fillDownLabelsDefault` leaves existing fields alone.

Harness changes:
- the case builder (Python and the Rust mirror, with golden parts) writes the x14 show-as modes, a field's
  `item_attrs` and a per-field `xml.extLst`, and lets `table` set `dataPosition`;
- the snapshot pivot reader no longer takes the x14 `<x14:pivotTableDefinition>` for the root, which had dropped a
  pivot from the sidecar.

Owners of the new off cells (decision 0245): T-4163 (show-as), T-4698 (X14F fillDownLabels) and the new T-4745
(gridDropZones and subtotalCaption).
