---
id: T-3854
title: "Probe GROUPBY's scalar text values and error-cell row_fields beside field_headers and as 1x1 arrays"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence settles whether GROUPBY's scalar-input `#VALUE!` rules (T-3837) hold beyond the cells that found them, so the
engine can apply them as widely as Excel does or no wider.

## Context
- T-3837 made GROUPBY `#VALUE!`, after the options' errors, for scalar text `values` and for a one-cell reference to an error
  as `row_fields`, but only with field_headers omitted: every scalar-pair cell of the gen-m7 arg-pairs batch (35679b4facc62956)
  omits it. Its reviewer (excel-fidelity, PR #2860) noted that the evidence also fits Excel guessing headers when field_headers is
  omitted (a lone text row taken as the header leaves no records). arrays.md "GROUPBY and PIVOTBY" has long said "Unprobed:
  whether one row of text values with no rows below is a header".
- Probe with `excel-probe` (one formula per row, ERROR.TYPE or TEXTJOIN of the result):
  - `GROUPBY(2,"abc",_xleta.COUNTA,f)` and `GROUPBY(Z4,Z2,..,f)` for f = 0, 1, 2, 3, with Z2 text and Z4 a number;
  - 1x1 arrays: `GROUPBY({"a"},{"x"},_xleta.COUNTA)`, `GROUPBY(2,{"abc"},..)`, `GROUPBY({#N/A},{1},_xleta.SUM)`, and a FILTER that
    returns one row with a text value;
  - `GROUPBY(Z1,Z4,_xleta.SUM,f)` with Z1 =1/0, for f = 0..3;
  - a 1x1 range (`Z3:Z3`, Z3 blank) as `values`, against `Z3`.
- Code: `Model::groupby_fields` / `ScalarInput` in crates/rustsheet-engine/src/functions/lookup_and_reference/groupby.rs.

## Acceptance criteria
- [ ] An excel-graph probe case covers the rows above; docs/excel-semantics/arrays.md records the rule and settles the
      "one row of text values" question.
- [ ] The engine matches every probe cell (widen or keep `ScalarInput`'s domain); `make check` green; reviewers approve.

## Out of scope
- PIVOTBY (T-3841).
