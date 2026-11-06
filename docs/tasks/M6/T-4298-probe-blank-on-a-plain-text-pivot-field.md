---
id: T-4298
title: "Probe \"(blank)\" on a plain text pivot field; scope the grouped-field guard; sync SPEC-0015 unknown 3 (T-4260 review follow-up)"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The record of GETPIVOTDATA's `"(blank)"` item rests on oracle cells for every kind of field the engine applies it to,
and SPEC-0015 agrees with docs/excel-semantics/web.md.

## Context
Follow-up of T-4260 (PR #3290). Its excel-fidelity review approved, with these MINOR findings:
- `Model::pivot_blank_text` (crates/rustsheet-engine/src/functions/lookup_and_reference/pivot_index.rs) lets
  `"(blank)"` pick the blank item of any ungrouped field. The only evidence is probes/t4236-pivot-date-groups, where
  every such field is a date field whose grouping Excel dropped. A plain text row field with a blank (as in the unit
  test `an_omitted_item_is_the_blank_item`) is inferred, not probed. The T-2662 entry of web.md ("GETPIVOTDATA on
  real-model pivots") says this without the caveat.
- That function's doc comment says `"(blank)"` is `#REF!` on a numeric range or date-grouped field. The cited probes
  (t4228-pivot-range-groups, t4236-pivot-date-groups-no-blank) have no blank in the grouped field, so they show only
  `#REF!` with no blank item. A numeric range field over a blank is unprobed.
- T-4260's Outcome says probes/t4236-pivot-date-groups went from 225 to 320 of 335 cells. It was 230 (105 known
  failures before, 90 fixed).
- docs/specs/SPEC-0015 (unknown 3, and the item rules near lines 54 and 183-186) still says every number or date
  item is `#REF!`, and names only the omitted item for the blank. T-4228, T-4246 and T-4260 resolved part of that.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph): GETPIVOTDATA with `"(blank)"`, `"(BLANK)"`, `""` and a blank
      cell on a plain text row field holding a blank, and on a numeric range-grouped field over a blank.
- [ ] web.md and the `pivot_blank_text` doc comment state what the probe shows; the engine follows it if it differs.
- [ ] T-4260's Outcome count is corrected to 230 -> 320, and SPEC-0015's unknown 3 and blank-item text match web.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The half-second rounding of pivot H (T-4261).
