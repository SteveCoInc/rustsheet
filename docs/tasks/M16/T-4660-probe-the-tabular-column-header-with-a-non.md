---
id: T-4660
title: "Probe the tabular column header with a non-compact column field, and outline alone on page and unused fields"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md settles the two parts of the tabular header rules T-4609 left open:
- whether the column header of a tabular pivot reads the column field's name (`Channel`) when that
  field is `compact="0"` but another field is left at the default, or `Column Labels`;
- whether `outline` alone (with `compact="0"`) on a page field or an unused field changes the row header.

## Context
probes/t4609-pivot-row-labels-2 (T-4609) showed `Column Labels` beside `Row Labels` when the column
field is at the default (L1!C3, L8!C3) and `Channel` beside `Region` when every field is
`compact="0"` (L7!C3), so it cannot tell "the column field's own `compact`" from "every field's".
`outline` alone was varied on the outer row field (L3) and the column field (L7) only, and on the data
field by t4123 T3 and T4. Variants: a tabular pivot with column field Channel `compact="0"` and the
inner row field (or an unused field) at the default; a page field, and an unused field, with
`compact="0"` and `outline` at the default. Case builder v2 with excel-graph.

## Acceptance criteria
- [ ] Filled probe with the variants; the pivots.md entries' UNVERIFIED (T-4660) parts are closed or corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine rendering (T-4149).
