---
id: T-4583
title: "Probe what decides Row Labels in tabular pivots, and colHeaderCaption with a real column field"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md states which non-axis or axis fields decide `Row Labels` versus the field name in a tabular pivot, and whether `colHeaderCaption` replaces `Column Labels` beside a real column field.

## Context
T-4137 (probes/t4137-pivot-captions K5, K6) and T-4123 (probes/t4123-pivot-capture T to T4) show that a default-compact non-axis field gives `Row Labels`. In every probe, though, that field was also the pivot's last field, and no probe left an axis field at the default. So "every field", "every non-axis field" and "the last field" all fit the evidence (Excel-fidelity review of T-4137). K1 set `colHeaderCaption` with only Values on columns, and it rendered nowhere. Build with case builder v2 and fill with excel-graph.

## Acceptance criteria
- [x] Filled probes: a default-compact unused field in the first and in a middle position, with the last field non-compact; an axis field at the default with every other field non-compact.
- [x] A filled probe with `colHeaderCaption` and a real column field.
- [x] pivots.md states the rule with this evidence, and the T-4137 entry's UNVERIFIED part is closed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4149 renders headers).

## Outcome
probes/t4583-pivot-row-labels (excel-graph, 2026-11-06) holds eight builder-v2 pivots. Recorded in
docs/excel-semantics/pivots.md:
- **Row Labels rule (R1–R6):** in a tabular pivot the first row header reads the field's name only
  when every pivotField is non-compact. Any one field at the default gives `Row Labels`, in every role
  probed (outer or inner row field, data, unused) and position (first, middle, last). This closes the readings
  T-4137 left open: "every field" holds, while "non-axis only" and "last field" are refuted by R1, R2,
  R3 and R4.
- **Outer field at the default (R1):** the inner field moves into the outer field's column, and the
  subtotals move onto the outer item's own line (no `<item> Total` lines).
- **Left open** (T-4609): column and page fields, `outline` alone on axis fields, and which attribute
  moves R1's subtotals.
- **colHeaderCaption (C1, C2):** in compact form it replaces `Column Labels` beside a real column field
  (C1!B3). In tabular form with every field non-compact both headers read the field names, and the
  caption is ignored (C2).
