---
id: T-4353
title: "Sheet-scoped defined name: resolve unqualified refs against the name's own sheet"
milestone: M12
lane: api
status: todo
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
- [ ] Oracle evidence recorded in docs/excel-semantics/ for the unqualified-ref case, and the facade matches it with a test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Python bindings (T-2900 documents the current behavior).
