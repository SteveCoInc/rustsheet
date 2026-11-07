---
id: T-4759
title: "Write x14 pivotShowAs on export when no dataField extension was kept, and validate eval spec show-as names with ShowDataAs"
milestone: M16
lane: engine
status: done
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
- [x] Exporting a pivot with an x14 show-as mode and no kept extension writes the x14
      `dataField` extension (or refuses the field with a documented reason); a round-trip
      test covers both the imported and the code-built case.
- [x] The eval spec builder validates names with `ShowDataAs` and its private lists are gone.
- [x] A unit test round-trips every `ShowDataAs` variant through `from_name(name())`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The import precedence of `showDataAs` versus `pivotShowAs` (UNVERIFIED, T-4750).

## Outcome
- `ShowDataAs::x14_data_field_extension(prefix)` (engine `pivot/model.rs`) is the one definition
  of the `<ext>` Excel writes for an x14 mode (uri `{E15A36E0-9728-4e99-A89B-3F7291B0FE68}`,
  `xmlns:x14`, `<x14:dataField pivotShowAs>` alone); `None` for an ECMA mode. OOXML in the engine
  is a deliberate exception: the xlsx exporter and the eval builder both reach it.
- Export (`export/pivots/show_as.rs`): the model's mode is the truth. The kept `<dataField>`
  elements are verbatim only when the `pivotShowAs` they name (the importer's
  `data_field_x14_show_as`, now shared by `read_extension`) is the model's x14 mode, or neither
  names one. Otherwise the kept `extLst` is rewritten: the first x14 `dataField` takes the mode
  (merged as an attribute, so one extension of that uri), others lose a stale one, an ECMA or
  `normal` mode removes it, and a list without one gains the extension under the list's own
  prefix; with no list kept, a list of its own. Kept XML it cannot read, or a prefixed
  `pivotShowAs`, is written as kept with nothing added; a rewrite must read back as the model's
  mode. `showDataAs` is never written for an x14 mode. The start-tag rewriter is
  `export::patch::rewrite_tag`, generalized from the cell patcher's `rewrite_start_tag`.
- Tests (`export/pivots/tests.rs`): an API-built field round-trips and its reload writes the kept
  extension back once; a kept extension is verbatim; a stale x14 mode is rewritten beside another
  extension; a kept x14 mode under `percentOfTotal` / `normal` is dropped and reloads equal; a
  kept attribute-only x14 `dataField` takes the attribute; prefixed lists (empty and not) export
  and reload; unreadable kept XML is written as kept.
- The eval spec builder's `SHOW_DATA_AS` / `SHOW_AS_X14` / x14 constants are gone: it validates
  with `ShowDataAs::from_name` and writes the engine's extension; the golden parts are unchanged.
  The Python builder is unchanged.
- `every_show_data_as_mode_round_trips_through_its_name` walks every variant through an
  exhaustive match, so a variant missing from `SHOW_DATA_AS_NAMES` fails it.
- UNVERIFIED (T-4948, pivots.md entry beside "Show-values-as modes as Excel for the web renders
  them"): the attribute-less `<x14:dataField/>` a stale-mode removal can leave, and the prefixed
  `<x:ext>` written into a prefixed kept list, are seen in no Excel-saved file; T-4948 probes both.
