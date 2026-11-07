---
id: T-4878
title: "A LiftArrays argument that IF, CHOOSE, INDEX(...,0,0) or LET picks a reference into another workbook through: CHOOSE still trips the scalar-context debug_assert"
milestone: M12
lane: engine
status: todo
priority: P0
size: S
tier: opus
depends_on: [T-4859]
---

## Goal
A `LiftArrays` argument (EDATE, the IM* functions, COMPLEX, ISEVEN, …) that reaches a reference
into another workbook through IF, CHOOSE, INDEX or LET reads it the way Excel does. In particular
`=IMAGINARY(CHOOSE(1,'[other.xlsx]Sheet1'!A1:B2))` no longer panics in a debug build.

## Context
These are follow-ups from T-4859's Excel-fidelity review (#3909). T-4859 fixed a written external
reference as a `LiftArrays` argument: several cells are refused (`#VALUE!`), one cell reads empty
(probes/external-ref-no-link-lift-arrays; references.md "An Analysis ToolPak argument refuses a
multi-cell reference into another workbook"). The reviewer checked the forms that pick such a
reference, on a debug build next to a `=SEQUENCE(2,2)` spill as in the fuzz model:
- **`IMAGINARY(CHOOSE(1,…!A1:B2))` panics.** It trips the same scalar-context `debug_assert!` in
  `Model::evaluate_cell` (model.rs) with a 2x2 array. This is a directive-5 crash, the same class
  as the 2026-11-05 `formula_eval` fuzz finding, so it is P0: the nightly fuzzer can find it next.
  IF is likely the same; check it.
- **`IMAGINARY(INDEX(…!A1:B2,0,0))` and `IMAGINARY(LET(a,1,…!A1:B2))` give `#BLOCKED!`.** They do
  not panic. A multi-cell INDEX over a local range is refused there (`#VALUE!`; references.md, the
  `LiftArrays` notes in `evaluate_as_reference`). INDEX is caught by the `FunctionKind` arm of
  `evaluate_as_reference` (functions/lift/call.rs) before the external arm.
- **A cached linked range** (`EDATE([1]Uses!$A$1:$A$3,1)`) is now refused. That follows from the
  rule but is not oracle-confirmed.

Probe first (`excel-probe`):
- IF, CHOOSE, INDEX(…,0,0), INDEX(…,1,1) and LET over `'[other.xlsx]Sheet1'!A1:B2` and `…!A1`
  as an EDATE and an IMAGINARY argument;
- ERROR.TYPE and ROWS wrappers, as in external-ref-no-link-lift-arrays.

Then make static analysis and the run-time read agree with Excel. The existing helpers are
`pick_reference` / `pick_branch` (external_reference.rs), `external_reference` and
`external_picked_value`. Optional, from T-4859's design review: one shared "pick one cell or
refuse" helper for `evaluate_as_reference`, the INDEX call sites (index.rs) and
`external_areas_value`, if it stays small.

## Acceptance criteria
- [ ] A probe case filled on excel-graph covers the forms above. Its finding is recorded in
      references.md.
- [ ] A debug-build unit test enters `=IMAGINARY(CHOOSE(1,'[other.xlsx]Sheet1'!A1:B2))` next to a
      spill without a panic, with Excel's value.
- [ ] The probed forms match Excel; the assert stays.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An oracle check of cached linked ranges, unless a fixtures workbook with an externalLink cache
  can be built for the probe.
