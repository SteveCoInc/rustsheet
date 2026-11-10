---
id: T-5039
title: "GETPIVOTDATA names a calculated data field by its name or caption (T-2085 LY G1:G3, moved from T-4804)"
milestone: M6
lane: functions-lookup
status: done
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
- [x] The cells above pass, and no other case regresses (`make eval-baseline`).
- [x] SPEC-0015's GETPIVOTDATA rules and web.md's "Ours" line say what the engine does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items (Excel for the web refuses them).

## Outcome
- `pivot_answer` (pivot_index.rs) answers a calculated data field with `calculated_answer`
  (pivot/calculated.rs): the field's `Plan` (the render's own compiler and sandbox, T-4174) is
  evaluated over the Sums of the fields its formula names, accumulated with the render's
  `Accumulator` over the records the pairs pick, whatever the data field's function. A refused
  formula or a call past `MAX_PIVOT_WORK` is `#REF!`. A unit test covers the sums, text skipped,
  no record, a refused formula and a field that is not calculated.
- SPEC-0015 (unknown 12, the import list) and web.md's "Ours" line describe it.
- Eval delta: probes/t2085-getpivotdata-layouts Q!G1:G3 fixed (probes known failures 10,265 to
  10,262); 0 regressions in the cell suites. Locally the roundtrip-calipers check reported
  pivot_named_range (two files) and two pv26 timeouts; neither workbook has a calculated field or
  a GETPIVOTDATA, so CI's run decides them.
- Later (T-4154, the M6 probe-cell PR): GETPIVOTDATA reads the pivot's `PivotResult`, whose calculated values are the
  render's own, so `calculated_answer` was removed; LY Q!G1:G3 still pass.
