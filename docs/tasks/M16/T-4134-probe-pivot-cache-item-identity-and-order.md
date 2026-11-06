---
id: T-4134
title: "Probe pivot cache item identity and order"
milestone: M16
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
docs/excel-semantics/pivots.md records how Excel builds a field's items on refresh: case-variant text (one item? which casing shows: first seen?), type order (numbers, text, booleans, errors, blank), dates with and without times, numbers that differ in the last bits, `(blank)` placement, and text that looks like a number.

## Context
libreoffice.md §2 (case-insensitive dedup, numbers < text < errors < blank) and oss-engines.md (gridcraft's class order) are hypotheses to confirm or refute. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Probe cases with sidecars covering each question; semantics entry with evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4134-pivot-items (excel-graph, 2026-11-06), five builder-v2 pivots, settles item identity and
order for docs/excel-semantics/pivots.md:
- **Identity:** texts differing in case are one item with the first-seen casing (I1). Numbers compare
  exactly (I4, 0.3 and 0.30000000000000004 are two items), and the text `"1"` is not the number 1 (I5).
- **Order:** Excel keeps a field's listed item order (I2), and appends items only refresh finds in
  source order (I3).
- **Rendering:** number items render as numbers, booleans as `TRUE`/`FALSE`, errors as their text and
  blanks as `(blank)`.

**Left open:**
- which casing wins when the cache and the source disagree;
- dates with and without times.

The case-variant refusal and the flag rules stay builder observations (T-4122 runs): a file Excel
refuses cannot be committed as a filled probe.
