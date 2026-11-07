---
id: T-4759
title: "Write x14 pivotShowAs on export when no dataField extension was kept, and validate eval spec show-as names with ShowDataAs"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4163]
---

## Goal
A pivot data field whose show-values-as mode is one of the six x14 modes is exported with
the mode intact even when it was not imported (no kept dataField extension), and the eval
spec builder validates show-as names through the engine's single `ShowDataAs` name table.

## Context
Follow-ups from the T-4163 review (fidelity and design MINORs):
- `crates/rustsheet-xlsx/src/export/pivots/table.rs` writes `showDataAs` only for ECMA
  modes and relies on the verbatim-kept `<dataField>` x14 extension for the x14 modes. A
  `DataField` given an x14 mode in code (no kept extension) exports silently as `normal`.
  Excel writes an x14 mode as `<x14:dataField pivotShowAs="...">` alone (pivots.md
  "Show-values-as modes as Excel for the web renders them").
- `crates/rustsheet-eval/src/pivot_definition.rs` keeps its own `SHOW_DATA_AS` /
  `SHOW_AS_X14` name lists; `ShowDataAs::from_name` / `is_x14` (engine `pivot/model.rs`,
  `SHOW_DATA_AS_NAMES`) is the one definition (CLAUDE.md directive 3).
- `ShowDataAs::name()` falls back to `"normal"` via `map_or`; a round-trip test over every
  variant would catch a variant missing from the table.

## Acceptance criteria
- [ ] Exporting a pivot with an x14 show-as mode and no kept extension writes the x14
      `dataField` extension (or refuses the field with a documented reason); a round-trip
      test covers both the imported and the code-built case.
- [ ] The eval spec builder validates names with `ShowDataAs` and its private lists are gone.
- [ ] A unit test round-trips every `ShowDataAs` variant through `from_name(name())`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The import precedence of `showDataAs` versus `pivotShowAs` (UNVERIFIED, T-4750).
