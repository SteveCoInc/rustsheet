---
id: T-3897
title: "R1C1 typed entry: a LET name spelled like an A1 cell (f1) is accepted"
milestone: M2
lane: engine
status: in-progress
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
- [ ] A probe with excel-graph values for cell-shaped LET and LAMBDA names typed in R1C1 mode; the engine matches.
- [ ] docs/excel-semantics/syntax.md "Short R1C1 forms typed in R1C1 mode" records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A1-mode typed entry (decision 0103 stands).
