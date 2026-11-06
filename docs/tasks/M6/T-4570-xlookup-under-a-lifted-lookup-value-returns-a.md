---
id: T-4570
title: "XLOOKUP under a lifted lookup_value returns a range if_not_found as a reference element (implement the mixed-array spec)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
campaign: semantics
depends_on: [T-4569]
---

## Goal
`XLOOKUP` with an array lookup_value and a range if_not_found returns Excel's mixed array: a value per hit and the
range itself, as a reference, per miss. Every reader of probes/t4563-xlookup-not-found-ref-element sees what Excel
sees.

## Context
- Evidence: lookup.md "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array" (T-4543, T-4563), with
  probes/t4543-xlookup-not-found-address-sheet-lift and probes/t4563-xlookup-not-found-ref-element (excel-graph).
- The representation and each reader's rule come from SPEC-0014's amendment "Mixed grids, value elements beside
  references" (T-4569; docs/specs/SPEC-0014-reference-array-results.md). Today
  `xlookup.rs` `xlookup_if_not_found` gives `#VALUE!` for a range under a lifted lookup_value.
- probes/t4563 A49 and A53 (`CELL("address",..)` of a reference on Data) include Excel's workbook name in brackets,
  which is the oracle file's hashed name. Check how other CELL("address") cases score that prefix before matching it. Never match the hash itself.

## Acceptance criteria
- [x] SPEC-0014 "Mixed grids, value elements beside references" is implemented as written: `RefElement::Value`,
      XLOOKUP's reference form, and each reader arm (value view, INDEX, the top-left and not-a-number aggregates, the
      `#N/A` order statistics, per-element callers). Deviations from the spec are amended into it in the same PR.
- [x] The visibly-off cells named below pass, with no regression.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Owner of the visibly-off cells of probes/t4543-xlookup-not-found-address-sheet-lift (W45, W46) and of
probes/t4563-xlookup-not-found-ref-element (A1, A5, A9, A13:A14, A17, A21:A22, A29:A30, A33, A37, A41, A45, A49, A53, A73, A109, A113, T13, T17, T21, T29, T33, T45, T49, T53, T73, T109, T113, U13, U17, U21, U29, U33, U45, U49, U53, U73, U109, U113, V1, V5, V9, V13, V17, V21, V25:V26, V29, V33, V37, V41, V45, V49, V53, V73, V109, V113, W13, W17, W21, W29, W33, W45, W49, W53, W73, W97, W101:W102, W109, W113, X13, X17, X21, X29, X33, X45, X49, X53, X73, X109, X113, Y13, Y17, Y21, Y29, Y33, Y45, Y49, Y53, Y73, Y109, Y113, Z13, Z17, Z21, Z29, Z33, Z45, Z49, Z53, Z73, Z109, Z113): Excel's reference element and its readers, where ours is
`#VALUE!`.

- Also owner of the visibly-off cells of probes/t4569-xlookup-ref-element-max-min: Calc A1:A8, A11:A15, A17,
  A20:A24, A26, A31:A32, A35, A40:A44, A50:A53, A56:A60, A63, B20:B24, B26, B31:B32, B34, C20:C23, C31:C32, D20:D28,
  D31:D34, and Local E4:E5, F6, F9. Keep the same-sheet `#VALUE!` of LARGE, SMALL and MEDIAN (t2065-lift-*, t4569 Local
  E1:E3, F1:F5, F7:F8). Amended by T-4697: for a grid with references on both sheets the sheet of the FIRST reference
  element decides (own sheet `#VALUE!`, another `#N/A`, probes/t4697-large-small-median-mixed-sheet-refs).

## Out of scope
- An array (not a range) if_not_found, which takes its first element and matches today.

## Outcome
SPEC-0014's "Mixed grids" amendment is implemented. `RefElement::Value` holds a scalar beside the reference elements,
and the driver keeps a kernel's value instead of `#VALUE!`. `Model::xlookup_reference` is XLOOKUP's reference form:
under a lifted lookup_value with a range if_not_found, each miss is that range as a reference element, and the lift is
charged as the value lift is (`lift_references_costed`). The readers each take one arm: the value view (`as_node`),
INDEX, the top-left walker (a value element is an array element), SUMPRODUCT, the per-element callers (the value
element is a scalar argument), OFFSET and FORMULATEXT over a grid. LARGE, SMALL and MEDIAN are `#N/A` when a
reference element is on another sheet (`RefGrid::order_statistic_error`). This also fixes the OFFSET and INDIRECT
cells of t4569 (A31, A32, A40:A44). An XLOOKUP whose if_not_found is a reference is no longer a keyed (tailed) lookup,
so its whole-column form builds the grid in full. `test_tailed_array.rs` moves that expression to the not-tailed list
and keeps a literal if_not_found in the tailed one. The static analysis gives a per-element call over such an XLOOKUP
the lookup_value's shape (t4569 A26 `#SPILL!`).

Departures from the spec are written into its new "As implemented (T-4570)" section. One-index INDEX into a column grid
is a 1x1 array of the element's value view: t4563 T25:U25 and W25:Z25 match, but V25:V26 (the range handed back) do
not, which T-4701 owns.

Eval delta: probes known failures 9108 -> 8932 (176 fixed: t4543 2, t4563 104, t4569 70), 0 regressions in any suite.
Of the owned visibly-off cells, all pass but t4563 V25:V26. The t2065-lift-* and t4569 Local E1:E3, F1:F5, F7:F8
`#VALUE!` cells keep passing. Unit tests: `test_xlookup_ref_element.rs` (readers, order statistics by sheet, an
all-hit lift builds no grid, the 32-byte size bound).
