---
id: T-3897
title: "R1C1 typed entry: a LET name spelled like an A1 cell (f1) is accepted"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`=LET(f1,5,f1+1)` typed in R1C1 mode (`entered_r1c1`) is 6, as in Excel, where the engine now refuses it.

## Context
- Found by T-3212. Typed in R1C1 mode, `f1` is no reference, and Excel accepts it: in a draft of
  probes/t3212-r1c1-bare-offsets (excel-graph, 2026-11-03) `=LET(f1,5,f1+1)` in S!H34 was 6, FORMULATEXT
  `=LET(f1,5,f1+1)`. The row was dropped from that probe because the case errors out.
- `rustsheet_engine::r1c1_formula_to_a1` (typed_input.rs) returns the A1 text, and the eval adapter
  (crates/rustsheet-eval/src/engine.rs `r1c1_entry`) types that text in A1 mode, where decision 0103 refuses a
  cell-shaped LET name (`Formula: the LET or LAMBDA name 'f1' is spelled like a cell`). That is not a
  `TYPED_FORMULA_DOES_NOT_PARSE` refusal, so the case fails with an engine error.
- Fix in the entry path: the R1C1 parse already applies R1C1's typed refusals (T-3212). The A1 text it hands on
  should then bind `f1` as stored text does, for example with `_xlpm.` names or a typed-entry flag that skips the
  A1 cell-shape check. Probe the case again (a new probe, `entered_r1c1` `=LET(f1,5,f1+1)` and a LAMBDA
  parameter `a1`).

## Acceptance criteria
- [x] A probe with excel-graph values for cell-shaped LET and LAMBDA names typed in R1C1 mode; the engine matches.
- [x] docs/excel-semantics/syntax.md "Short R1C1 forms typed in R1C1 mode" records it.
- [x] `make check` green; reviewers approve.

## Out of scope
- A1-mode typed entry (decision 0103 stands).

## Outcome
- New probe probes/t3897-r1c1-let-cell-names (excel-graph, 2026-11-04), 13 formulas typed in R1C1 mode with
  ISBLANK and FORMULATEXT. Excel accepts a LET name or LAMBDA parameter spelled like an A1 cell in any letter case
  (`f1`, `F1`, `a1`, `A1`, `xy1`, `ab12`, an optional `[b2]`, a called `f1`), rows 1 to 7, 11 and 12. Unbound, `f1`
  or `F1` is an undefined name: `#NAME?`, and FORMULATEXT shows it lower-cased and quoted, `='f1'` (rows 8 to 10, 13).
  Both recorded in syntax.md "Short R1C1 forms typed in R1C1 mode".
- `r1c1_formula_to_a1` now writes a bound cell-shaped name `_xlpm.f1` in the A1 text
  (`stringify::bound_variables::to_english_string_typed_from_r1c1`), which the typed A1 entry binds where it refuses
  a bare `f1` (decision 0103 stands for A1-mode typing). `walk_names` visits LAMBDA parameters now
  (`NameSite::Parameter`), and the export's `prefix_bound_variables` skips them, since the export prints their prefix.
- The R1C1 lexer read an upper-case cell-shaped name (`F1`, `A1`) as an illegal identifier, so `=LET(F1,5,F1+1)`
  typed in R1C1 mode was refused. It now reads it as a name, as it did lower case (`utils::is_name_text`).
- An unbound cell-shaped name is refused (blank cell) rather than written bare, which the A1 entry would read as the
  cell. Before, a lower-case `=f1` read the cell F1. Excel's `#NAME?` and `='f1'` are T-4040 (filed).
- Eval delta: no gated cell fixed or regressed in any suite. The new case passes 27 of 39 cells. Its 12 misses
  (A8:C10, A13:C13) are new-coverage known failures, all owned by T-4040.

