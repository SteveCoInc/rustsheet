---
id: T-4805
title: "GETPIVOTDATA: number, date and boolean items, a repeated field, text, blank and error records, error arguments, array data_field and field name (T-2085 evidence)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
depends_on: []
---

## Goal
GETPIVOTDATA matches Excel on number, date and boolean items, a field given twice, records that are text, blank or errors, error arguments, and array `data_field` and field-name arguments.

## Context
T-2085 probed SPEC-0015 unknowns 3, 5, 6, 7, 13 and 14 (docs/excel-semantics/web.md "GETPIVOTDATA on other pivot shapes (T-2085)", the "Number, date and boolean items", "A repeated
field", "Records that are not numbers", "Errors in the arguments" and "Arrays" paragraphs). An array data_field or
field name lifts per SPEC-0011 and spills, as an array item already does.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-items Q!A1:A6, Q!A8, Q!A9, Q!B3, Q!C1:C3, Q!C5, Q!E1, Q!E2, Q!E9, Q!E12:E15, Q!E17, Q!F9,
  Q!F10, Q!E20:E23

An error cell of the pivot given as pivot_table only picks the pivot (Q!E20:E23). Q!E11 passes today only because
the pivot's grand total is the same `#N/A` that the engine passes through. An error as a field name (Q!E3) already
passes: T-4823 implemented web.md "GETPIVOTDATA error arguments (T-4006)", which agrees with this probe.

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] SPEC-0015's resolution rules and web.md's "Ours" line say what the engine now does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The shown text of date items in the pivot itself (T-4807).

## Landing notes
- 2026-11-07 opus run: claimed at minute ~65, no code written; the session's landing window closed first. No PR.
- Start after PR #3950 (T-4804) merges: it rewrites `getpivotdata_on` (data fields, the Values pair) in
  functions/lookup_and_reference/getpivotdata.rs, which this task's item matching changes too. Branch from the
  `origin/main` that has it.
- Findings so far (probes/t2085-getpivotdata-items.case.json, 103/130): number, date and boolean items fall to
  `_ => None` or `pivot_number_item` misses (Q!A1:A9, B3, C1:C5); a repeated field returns `None` at
  `positions.contains` (E1, E2); the Odd pivot (text, blank and error records in the data field) is `#REF!`
  throughout (E12:E17), likely `record_numbers` refusing a non-number record; a single error cell as pivot_table is
  passed through at the `left == right` check in `fn_getpivotdata` (E20, E23); array data_field / field name (E9:F10)
  are not lifted.
