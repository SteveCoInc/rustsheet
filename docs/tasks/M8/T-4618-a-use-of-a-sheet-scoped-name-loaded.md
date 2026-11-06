---
id: T-4618
title: "A use of a sheet-scoped name loaded before its API add resolves in probes/t4554-api-name-local-name-body Last!D3"
milestone: M8
lane: api
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Find what makes Excel resolve probes/t4554-api-name-local-name-body Last!D3 (`=LD` on Last, loaded before `LD` is
added through the API scoped to Last) to the new name (90), where the rule "A sheet-scoped name added after load"
(docs/excel-semantics/syntax.md, T-4049) leaves such a use `#NAME?`, and make the engine match.

## Context
- Evidence: probes/t4554-api-name-local-name-body (excel-graph 2026-11-06): tabs First, S2, Last; S2 active; the file
  holds `S2!LocLoc`, a workbook `Dup` and `S2!Dup`, `Last!Dup`; API adds in order `WL` = `=LocLoc`, `WD` = `=Dup`
  (workbook), then `LD` = `=Dup` on Last. Last!D3 `=LD` is 90; First!D3 and S2!D3 `=Last!LD` are 90 too.
- The rule holds against four probes of T-4587 (excel-graph 2026-11-06), each `#NAME?` on the scoped sheet:
  t4587-sheet-api-name-inactive-sheet (other sheet active), t4587-sheet-api-name-sheet-has-names (the sheet already
  has a file sheet-scoped name), t4587-sheet-api-name-name-body (body is a bare name), and
  t4587-sheet-api-name-qualified-use-first (an earlier tab holds `=S!N`).
- Differences not yet probed: workbook-scoped API adds earlier in the same session (`WL`, `WD`); the scoped sheet
  being neither the first tab nor active with three sheets; the body naming a name the same sheet defines (`Dup`
  on Last). Probe each in its own case (one variable per case) before changing code.
- Ours: `Model::bind_added_sheet_name` (crates/rustsheet-engine/src/defined_names.rs) pins the unresolved use to
  `[0]!LD`, so Last!D3 is `#NAME?`.

## Acceptance criteria
- [x] probes/t4554-api-name-local-name-body Last!D3 passes, or a probe shows the cell depends on something the harness
      cannot reproduce (record it in syntax.md and leave it in the baseline).
- [x] The new probes' finding is in docs/excel-semantics/syntax.md "A sheet-scoped name added after load".
- [x] `make check` green; reviewers approve.

## Out of scope
- Bare-name binding inside API-added bodies (T-4587).

## Outcome
- Probed (excel-graph 2026-11-06): 23 new probes/t4618-* cases in four rounds, one variable per case.
  - Workbook adds earlier in the session, the body and the sheet's own names change nothing.
  - The tab layout and the active sheet do. They show a load-order model, recorded in docs/excel-semantics/syntax.md
    "Which place a loaded use took": Excel loads the defined names, then the active sheet, then the other tabs in
    order; qualified and bare uses make own and provisional places; the add fills S's own place, or S's unfixed
    provisional place.
  - The model fits every bare-use cell of the 30 related probes. That includes t4394's `=SUM(Span)` and `=Rate2`,
    which T-4391 left open.
  - One qualified cell is unexplained: t4618-order-after-active-backward Q!A1, to T-4642.
- Engine changes:
  - `Model::loaded_uses_take_sheet_place` (new name_load_order.rs) replays the model over the stored formulas.
  - `Model::bind_added_sheet_name` skips pinning when the add fills the place.
  - `name_load_events` (name_binding.rs) reads a formula's uses in written order, through `walk_names_in_order` and
    `chain_operands_in_order_mut`, without recursion over chains.
  - `walk_names` now also visits defined-name nodes as `NameSite::DefinedName`.
- Tests: three unit tests in test_sheet_name_added_binding.rs.
- Eval delta: 8 cells newly pass (t4554-api-name-local-name-body Last!D3; t4394-api-name-active-last S2!A1, S2!A3,
  Last!A1; t4394-api-name-active-s2 S2!A3, Last!A1; t4353-api-name-unqualified-ref S2!A3, Last!A1). 23 cases were
  added (138 cells), all passing but t4618-order-after-active-backward Q!A1 (T-4642). No regressions.
