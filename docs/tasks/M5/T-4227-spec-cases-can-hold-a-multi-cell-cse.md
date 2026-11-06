---
id: T-4227
title: "Spec cases can hold a multi-cell CSE array formula (array_formulas), written by spec2xlsx and entered by the engine adapter"
milestone: M5
lane: harness
status: todo
priority: P3
size: M
tier: sonnet
depends_on: []
---

## Goal
A probes case can hold a legacy CSE array formula entered over a multi-cell area (`{=f}` over `B1:B3`), so that
Excel's behavior on the other cells of such a block can be probed on excel-graph and scored against the engine.

## Context
- Split out of T-4226 (its handoff): the case format writes only single-anchor dynamic-array formulas
  (`spec2xlsx.py` calls `write_dynamic_array_formula(row, col, row, col, ..)`) and single-cell `legacy_formulas`; no key
  writes a multi-cell `<f t="array" ref="B1:B3">`, and `rg array_formulas` finds nothing.
- Suggested shape: an optional per-sheet `"array_formulas": {"B1": "B1:B3"}` (anchor, which must be a formula cell in
  `cells`, → its area, which must contain the anchor at its top-left; the other cells of the area must be absent from
  `cells`). Validate both in `case.rs` and in `spec2xlsx` (reject overlap with `legacy_formulas`,
  `bare_function_names` and other areas; bound the area like `set_user_array_formula`'s 2^20-cell refusal).
- Writing: `spec2xlsx` writes it with xlsxwriter's `write_array_formula(first_row, first_col, last_row, last_col, ..)`
  (a legacy, non-dynamic array formula). The oracle uploads the built file, so excel-graph needs no change; check that
  the excel-desktop and libreoffice backends either handle it or refuse the case explicitly.
- Entering: the engine adapter (`crates/rustsheet-eval/src/engine.rs`, `cell_input`) has no facade input for a CSE
  area: `rustsheet::Input` (`crates/rustsheet/src/edit.rs`) has no array variant, while the engine has
  `Model::set_user_array_formula` / `UserModel::set_user_array_formula`, which store the other cells as
  `Cell::SpillCell`. Either add a facade entry point (api-lane change; keep it small and documented) or reuse whatever
  xlsx import uses for `<f t="array">` (`crates/rustsheet-xlsx/src/import/sheet_data.rs`), whichever keeps one
  definition. The formula is entered as stored (legacy: implicit intersection as for `legacy_formulas`, T-1096).
- Document the key in `docs/architecture/eval-harness.md` next to `legacy_formulas`.

## Acceptance criteria
- [ ] A per-sheet `array_formulas` key is parsed by `crates/rustsheet-eval/src/case.rs` (unknown/invalid areas are
      case errors, not panics), written by `spec2xlsx` as `<f t="array" ref=..>`, and entered by the engine adapter so
      that the area's other cells are array members of the anchor.
- [ ] Tests: a spec2xlsx unit test of the written XML, and an engine-adapter test that `{=ROW(A1:A3)}` over `B1:B3`
      evaluates to 1, 2, 3 with B2:B3 members of B1's block.
- [ ] `docs/architecture/eval-harness.md` documents the key.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The SUBTOTAL probe itself (T-4226).
