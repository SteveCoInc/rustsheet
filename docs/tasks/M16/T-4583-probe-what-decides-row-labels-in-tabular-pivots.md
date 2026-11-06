---
id: T-4583
title: "Probe what decides Row Labels in tabular pivots, and colHeaderCaption with a real column field"
milestone: M16
lane: research
status: todo
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
- [ ] Filled probes: a default-compact unused field in the first and in a middle position, with the last field non-compact; an axis field at the default with every other field non-compact.
- [ ] A filled probe with `colHeaderCaption` and a real column field.
- [ ] pivots.md states the rule with this evidence, and the T-4137 entry's UNVERIFIED part is closed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4149 renders headers).
