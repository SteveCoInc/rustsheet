---
id: T-5039
title: "GETPIVOTDATA names a calculated data field by its name or caption (T-2085 LY G1:G3, moved from T-4804)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4804]
---

## Goal
GETPIVOTDATA answers a calculated data field by its name or its caption, as Excel does: `"Double"` is 92 and 42, and
`"Sum of Double"` with West is 50 (web.md "GETPIVOTDATA on other pivot shapes (T-2085)", "Calculated fields").

## Context
- T-4174 (#3945) loads and renders a cache with a calculated field and moved these cells to T-4804. T-4804's PR
  (#3950) was written before that move and does not implement them: GETPIVOTDATA aggregates cache records
  (`pivot/layout.rs` `record_numbers`, `pivot_index.rs`), and a calculated field has no records, so it answers `#REF!`.
  Its value is the calculated field's formula over the aggregated source fields, as the pivot render computes it
  (`pivot::result`, T-4174): reuse that, do not re-derive it.
- Owner of these visibly-off cells (decision 0245):
  - probes/t2085-getpivotdata-layouts Q!G1:G3

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] SPEC-0015's GETPIVOTDATA rules and web.md's "Ours" line say what the engine does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items (Excel for the web refuses them).
