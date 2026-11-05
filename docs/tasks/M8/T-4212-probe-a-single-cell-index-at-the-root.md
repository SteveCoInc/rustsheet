---
id: T-4212
title: "Probe a single-cell INDEX at the root over a union, a sheet-qualified range, a 2-D range and a reference-bodied name"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for whether a legacy single-cell INDEX shows no `@` at the root (and under ABS) over every area kind the
engine's scalar INDEX shape accepts, and the engine matching it.

## Context
- T-3856 (PR #3180) made `INDEX(area,<literal whole numbers>)` a scalar shape (`indexes_one_reference_cell`,
  static_analysis/if_arm.rs) when the area is a range, a union or a name whose body is a cell or range reference.
  Root evidence (probes/t3856-legacy-at-name-body row 45) covers the plain range `Z1:Z6` only; the other area kinds rest
  on IF-arm evidence (probes/t3134-legacy-if-index-arm-shapes, t3297-legacy-if-index-arm-more-shapes and -name-table).
- Excel-fidelity review MINOR on PR #3180.

## Acceptance criteria
- [ ] A probe (excel-graph) with legacy formulas and FORMULATEXT beside each: `=INDEX((B1:B6,C1:C6),3,1)`,
      `=INDEX((B1:B6,C1:C6),3,1,2)`, `=INDEX(Sheet1!B1:B6,3)`, `=INDEX(B1:C6,3)`, `=INDEX(B1:C6,3,1)`, `=INDEX(Rng,3)`
      (Rng = Sheet1!$B$1:$B$6), `=INDEX(One,1)` (One = Sheet1!$B$3), each at the root and under ABS, plus the typed
      (dynamic) twins' values.
- [ ] The engine matches every cell, or the visibly-off cells have an owner task; references.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INDEX over array-bodied names (kept as before T-3856).
