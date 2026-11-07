---
id: T-4730
title: "Desktop: pivot flags Excel for the web refuses (dataSourceSort, item d, item f)"
milestone: M14
lane: research
status: dropped
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Find out what desktop Excel does when it opens a pivot that Excel for the web refuses: it opens it, repairs it
(and drops what), or refuses it. The three catalog rows are `pivotField@dataSourceSort="1"` (PV-1095),
`item@d="1"` (PV-1076) and `item@f="1"` (PV-1078, with no `calculatedItems`).

## Context
- T-4129 (oracle capability matrix 1) ran each flag on a pivot of its own (excel-graph, 2026-11-06). Every
  other flag of the layout, totals, aggregation and show-values-as families opened and rendered. On these three,
  createSession returned 501 `FileCorruptTryRepair` (inner code `unsupportedWorkbook`). Excel saves nothing for
  a file it refuses, so no sidecar exists.
- The definitions:
  - `dataSourceSort`: probes/t4129-pivot-layout-flags FA, with `dataSourceSort` added to Region's attrs;
  - `d` and `f`: probes/t4129-pivot-item-flags, with `d` or `f` in place of the case's flag on Product item B.
- See docs/excel-semantics/pivots.md, "Pivot flags Excel for the web refuses".
- Decision 0274 sends the pivot rows the web cannot render to M14, and they never block M16.

## Acceptance criteria
- [ ] With an excel-desktop backend, each of the three pivots is opened. The result (opens, repairs and drops
  what, or refuses) is recorded in pivots.md and in the catalog's evidence column.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items (`calculatedItems` with `f="1"` items): capability matrix 2 (T-4130).

## Outcome (dropped, 2026-11-07, decision 0266)
Dropped when filed. Desktop-oracle work is out of scope (decision 0266), and no other oracle can answer. The rows
stay `web=no` in eval/catalog/pivot-features.tsv. The context above is kept in case a later decision brings the
desktop backend back (0266 item 4).
