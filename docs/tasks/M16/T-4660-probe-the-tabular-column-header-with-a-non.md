---
id: T-4660
title: "Probe the tabular column header with a non-compact column field, and outline apart from compact on the inner row, page, unused and calculated fields"
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
- whether `outline` varied apart from `compact` changes the row header on the inner row field, a
  page field, an unused field or a calculated field, either way round: `compact="0"` with `outline`
  at the default (every other field non-compact, so the control reads the field name), and `compact`
  at the default with `outline="0"` (the control reads `Row Labels`).

## Context
probes/t4609-pivot-row-labels-2 (T-4609) showed `Column Labels` beside `Row Labels` when the column
field's `compact` is at the default (L1!C3, both attributes at the default; L8!C3, `outline="0"`)
and `Channel` beside `Region` when every field is `compact="0"` (L7!C3), so it cannot tell "the
column field's own `compact`" from "every field's". `outline` was varied apart from `compact` on the
outer row field (L3, L4) and the column field (L7, L8) only, and on the data field by t4123 T3 and
T4; never on the inner row field (Product is `compact="0" outline="0"` in every L sheet) or a
calculated field. Variants: a tabular pivot with column field Channel `compact="0"` and the inner
row field (or an unused field) at the default; the inner row field, a page field, an unused field
and a calculated field each with `compact="0"` and `outline` at the default, and each with `compact`
at the default and `outline="0"`. Case builder v2 with excel-graph.

## Acceptance criteria
- [ ] Filled probe with the variants; the pivots.md entries' UNVERIFIED (T-4660) parts are closed or corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine rendering and its eval effect (T-4152).
