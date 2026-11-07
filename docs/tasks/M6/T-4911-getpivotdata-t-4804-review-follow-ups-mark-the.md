---
id: T-4911
title: "GETPIVOTDATA T-4804 review follow-ups: mark the unprobed Values-field, blank-cell and nested-Values inferences, probe them, add a multi-data-field unit test"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every GETPIVOTDATA rule T-4804 (PR #3950) inferred without an oracle cell is either probed against Excel or marked
unprobed (UNVERIFIED) where it is stated, and the several-data-field path has a fast engine unit test.

## Context
T-4804's reviewers (design and Excel fidelity, all MINOR) found these inferences, which the code and
SPEC-0015 state as rules with no cell behind them. probes/t2085-getpivotdata-data-fields (DF) only covers the
default `"Values"` name and items written in the caption's own case.
- The Values pair field is matched by a custom `dataCaption`, and its item by the caption ignoring case. A Values
  name that collides with a cache field name is also unprobed (`getpivotdata.rs` `getpivotdata_on`; SPEC-0015
  GETPIVOTDATA step 3).
- A pivot with several data fields whose Values field shares an axis with another field is accepted and counts as
  nested (`pivot_index.rs` `supported`, `nested_axis`), so only its grand total is answered. That extends unknown 10's
  grand-total rule (calipers formula_stress_test, which has one data field) to an unprobed shape.
- Two blank-cell data_field cases are `#REF!` without saying so: a blank cell inside another pivot's area, and a
  blank cell in this pivot's page-field area (LY A5/A6 show a reference there picks the pivot).
- The comment in `pivot_data_field` cites DF A7, A14 and A16 for an omitted data_field too, but those cells only test
  `""`. The omitted form is still SPEC-0015 unknown 8.
- The several-data-field and Values-pair paths are covered only by the eval case, with no unit test in
  `test/test_fn_getpivotdata.rs`.
- The caption-versus-source-name precedence (caption wins) is already marked unprobed. It has no task id yet.

## Acceptance criteria
- [x] web.md "GETPIVOTDATA on other pivot shapes (T-2085)" "Ours" and SPEC-0015 mark each inference above
      UNVERIFIED with this task's id, or a new probe case (excel-probe skill) evidences it and the engine follows it.
- [x] The `pivot_data_field` comment cites unknown 8 for an omitted data_field.
- [x] A unit test in `test/test_fn_getpivotdata.rs` builds a two-data-field pivot with Values on the columns. It
      checks a source name shared by two data fields, a caption, the Values pair (matching and not) and `"Values"`
      as data_field.
- [x] `make check` green; reviewers approve.

## Out of scope
- T-4805 (items and arguments) and T-4806 (pivot shapes and shown values).

## Outcome
Marked every inference above UNVERIFIED (T-4911) in web.md "GETPIVOTDATA on other pivot shapes (T-2085)", SPEC-0015
and the `pivot_data_field` comment, which now cites unknown 8 for an omitted data_field. None was probed, so the
engine is unchanged. Added `several_data_fields_pick_by_source_name_caption_and_the_values_pair` (two data fields,
Values on the columns). No eval delta.
