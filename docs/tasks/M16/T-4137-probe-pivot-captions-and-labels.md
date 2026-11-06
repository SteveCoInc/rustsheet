---
id: T-4137
title: "Probe pivot captions and labels"
milestone: M16
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
Evidence for every caption Excel writes: Row Labels, Column Labels, Values, Grand Total, `<item> Total`, `Total <data field>`, `(blank)`, `(All)`, `(Multiple Items)`, the default data field name per aggregate (`Count of`, `StdDev of`...), duplicate data field names, and custom caption attributes (rowHeaderCaption, grandTotalCaption, dataCaption, missingCaption, errorCaption).

## Context
T-2327 also asked for the Count/Max/Min captions. gridcraft and mog claims in oss-engines.md. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Also: the first row header of a tabular pivot is `Row Labels` unless its data field is `compact="0"` (pivots.md, probes/t4123-pivot-capture); probe whether every data field must be non-compact when two source fields are data (probes/t4128-pivot-spike K keeps `Row Labels` with Sales non-compact and a calculated field at the default), with no filter or sort to confound it, and the layout without a Values axis.

## Acceptance criteria
- [x] Probe cases with sidecars; pivots.md entries.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4137-pivot-captions (excel-graph, 2026-11-06), six builder-v2 pivots:
- **Captions (K1–K3):** the caption attributes (row header, grand total, data caption), `missingCaption`
  and `errorCaption` all apply.
- **Page fields (K4):** `(All)`, a single item and `(Multiple Items)`.
- **Custom subtotals (K5):** labels read `<item> Sum` and `<item> Count`.
- **Row Labels rule, corrected (K5, K6):** an unused field at the default `compact` also gives
  `Row Labels`, so the rule is not "the data field's compact". Whether every field, every non-axis
  field or the last field decides is open (T-4583).
- **No Values axis (K5):** with one data field and no column field there is no Values row; `Row Labels` sits at the corner and the data caption above the data column.
- **`colHeaderCaption` (K1):** it renders nowhere when the column axis holds only Values; a real
  column field is T-4583's.

Recorded in docs/excel-semantics/pivots.md. K4 is consistent with the multi-page capture window, but
cannot confirm it (T-4138).
Default aggregate names are written by the file, so a file cannot probe them; the `Count of`/`Max
of`/`Min of` captions T-2327 asked about are not reachable this way, which is noted; T-2327 keeps
that question.
