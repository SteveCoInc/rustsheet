---
id: T-4040
title: "R1C1 typed entry: an unbound name spelled like an A1 cell (f1, F1) is #NAME?, FORMULATEXT ='f1'"
milestone: M2
lane: engine
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A name spelled like an A1 cell that no LET or LAMBDA binds, typed in R1C1 mode (`entered_r1c1` `=f1`, `=F1+1`),
is an undefined name as in Excel: `#NAME?`, and FORMULATEXT shows it lower-cased and quoted, `='f1'`.

## Context
- Found by T-3897. probes/t3897-r1c1-let-cell-names (excel-graph, 2026-11-04), with F1 holding 7: A8 `=f1`,
  A9 `=F1+1`, A10 `=LET(x,1,x+F1)` and A13 `=LET(f1,5,f1+1)+F1` are all `#NAME?`, not blank, and their
  FORMULATEXT (C8, C9, C10, C13) is `='f1'`, `='f1'+1`, `=LET(x,1,x+'f1')`, `=LET(f1,5,f1+1)+'f1'`: Excel
  lower-cases the name and quotes it like a sheet name.
- Today `rustsheet_engine::r1c1_formula_to_a1` (typed_input.rs) refuses such a formula
  (`stringify::to_english_string_typed_from_r1c1` returns `None`, `TODO(T-4040)`), so the cell is blank: bare
  A1 text would read the name as the cell F1.
- Carried from T-3897's review (PR #3102, MINORs):
  - Fidelity: passing the name on as an unbound `_xlpm.f1` may already give `#NAME?` (the A and B columns),
    leaving only FORMULATEXT's `'f1'` open. Try that first.
  - Fidelity: the doc comment of `bound_variables::reads_back_as_cell` still says the R1C1 lexer refuses a name
    spelled like an A1 cell as an identifier; since T-3897 it reads one as a name in any case. Correct it.
  - Design: `prefix_a1_cell_shaped_variables` repeats the A1 half of `reads_back_as_cell`
    (`parse_reference_a1(&name.to_ascii_uppercase())`); give both one small helper.
- Fix: carry the name through to the A1 entry as an undefined name (does the A1 parser read `'f1'` as one?
  probe it typed in A1 mode first), and print it `'f1'` in FORMULATEXT where Excel does.

## Acceptance criteria
- [ ] probes/t3897-r1c1-let-cell-names A8:C10 and A13:C13 match Excel.
- [ ] docs/excel-semantics/syntax.md "Short R1C1 forms typed in R1C1 mode" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LET and LAMBDA names spelled like a cell (T-3897).
