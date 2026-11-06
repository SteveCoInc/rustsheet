---
id: T-4609
title: "Probe Row Labels with column and page fields, and compact versus outline on an outer row field"
milestone: M16
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md settles three open parts of the tabular Row Labels rule:
- whether a column or page field left at the default `compact` also gives `Row Labels`;
- whether `outline` alone matters on an axis field;
- which of `compact` and `outline` on the outer row field moves its subtotals onto the item line (T-4583 R1).

## Context
T-4583 (probes/t4583-pivot-row-labels R1–R6) covered row, data, unused and calculated fields. R1 left both attributes at the default on the outer row field, so it cannot tell `compact` from `outline` (Excel-fidelity review of T-4583). Variants: a column field at the default; a page field at the default; an outer row field with `compact=0` and outline at the default, and the reverse. Also `rowHeaderCaption` with every pivotField at `compact="0"`: T-4149's renderer (`crates/rustsheet-engine/src/pivot/layout/render.rs`) writes the first row field's name, as C2 shows the column field's name over `colHeaderCaption` (UNVERIFIED in pivots.md "Rendered pivot cells"). Case builder v2 with excel-graph.

## Acceptance criteria
- [x] Filled probe with the four variants; the pivots.md entry's UNVERIFIED part is closed or corrected.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine rendering (T-4149).

## Outcome
probes/t4609-pivot-row-labels-2 (excel-graph, 2026-11-06) holds eight builder-v2 tabular pivots
(rows Region, Product; Sum of Sales; every field `compact="0" outline="0"` except as named).
Recorded in docs/excel-semantics/pivots.md:
- **Column and page fields count (L1, L2, L6):** a column or page field at the default gives
  `Row Labels`; the same page field non-compact gives `Region`. With the column field at the default
  its header reads `Column Labels` (L1!C3).
- **Only `compact` counts for the header, for the roles probed (L3, L4, L7, L8):** on the outer row
  and column fields, `outline` at the default with every field
  `compact="0"` still gives `Region` (L3 outer row field, L7 column field, whose header then reads
  `Channel`); `outline="0"` with `compact` at the default still gives `Row Labels` (L4, L8).
- **R1 split (L3, L4):** the outer row field's `outline` alone puts each item on its own line with
  its subtotal (L3, no `<item> Total` lines; L4 has them); the shared column needs `compact` and
  `outline` both on (L3 and L4 are A3:C11 with firstDataCol 2, R1 is A3:B11 with 1).
- **rowHeaderCaption (L5):** ignored when every field is `compact="0"`; A3 reads `Region`.
- The engine's `label_columns`, lines and T-4149's render already match all eight sheets, so no
  code changed: the probe joins the lines and render tests
  (crates/rustsheet-engine/src/pivot/layout/lines/tests.rs, render/tests.rs), and the UNVERIFIED
  T-4609 notes in lines.rs, render.rs and pivots.md are closed. The case is in T-4152's owner list.
- **Left open** (T-4660): the column header of a non-compact column field while another field is at
  the default, and `outline` varied apart from `compact` on the inner row field and on page, unused
  and calculated fields.
