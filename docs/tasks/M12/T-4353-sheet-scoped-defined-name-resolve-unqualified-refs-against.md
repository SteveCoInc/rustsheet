---
id: T-4353
title: "Sheet-scoped defined name: resolve unqualified refs against the name's own sheet"
milestone: M12
lane: api
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`Txn::define_sheet_name(sheet, "Rate", "=$B$1")` binds the unqualified reference to the name's own sheet, not the workbook's first sheet.

## Context
Found in T-2900: with Sheet2 holding B1=7, a sheet-scoped `Rate` defined as `=$B$1` on Sheet2 read Sheet1!B1 (5) from a Sheet2 formula; the qualified `=Sheet2!$B$1` gives 7. Excel files always store such formulas qualified, so first probe what the oracle does for an API-entered unqualified ref (excel-probe skill) before changing `crates/rustsheet/src/edit.rs` `add_defined_name`.

## Acceptance criteria
- [x] Oracle evidence recorded in docs/excel-semantics/ for the unqualified-ref case, and the facade matches it with a test.
- [x] `make check` green; reviewers approve.

## Out of scope
- Python bindings (T-2900 documents the current behavior).

## Outcome
- Excel does not do what the goal assumed: an unqualified reference in a name added through the API binds to the
  first sheet, whatever the name's scope (probes/t4353-api-name-unqualified-ref, excel-graph: a sheet-scoped `Rate`
  = `=$B$1` on S2 reads First!B1, 1, not S2!B1, 7; a range body and a workbook-scoped name alike). The facade already
  does this, so there is no code change: the quirk is recorded in docs/excel-semantics/syntax.md "An unqualified
  reference in an API-defined name binds to the first sheet", and the test
  `an_unqualified_name_reference_binds_to_the_first_sheet` (crates/rustsheet/src/edit/tests.rs) pins it.
- First sheet or active sheet: not told apart (First is both in the probe's workbook).
- The probe also shows uses typed before a sheet-scoped add resolving where T-4049's rule says they stay `#NAME?`
  (S2!A3, Last!A1): filed as T-4391.
- Eval delta: new coverage 1 case (8 checked cells: 6 pass, 2 known failures owned by T-4391), regressions 0.
