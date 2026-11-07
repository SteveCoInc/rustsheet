---
id: T-4040
title: "R1C1 typed entry: an unbound name spelled like an A1 cell (f1, F1) is #NAME?, FORMULATEXT ='f1'"
milestone: M2
lane: engine
status: done
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
- [x] probes/t3897-r1c1-let-cell-names A8:C10 and A13:C13 match Excel.
- [x] docs/excel-semantics/syntax.md "Short R1C1 forms typed in R1C1 mode" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LET and LAMBDA names spelled like a cell (T-3897).

## Outcome
- Probed first (probes/t4040-r1c1-unbound-cell-names, excel-graph, 2026-11-04): `='f1'` is refused typed in A1 and
  R1C1 mode (A1:A6 blank), so the quote is display only; the lower case in t3897 is the workbook's first spelling
  (`=AB12*2` shows `='AB12'*2`); every position quotes (`SUM('f1',2)`, `'f1'(2)`, a LAMBDA body, ISREF FALSE); a
  name that is no cell stays bare (`Xfe1`). A file's unbound `_xlpm.f1` is `#NAME?` too but FORMULATEXT keeps the
  prefix (A14:C16): filed as T-4099.
- `Parser::parse` gives an unbound name spelled like an A1 cell the quoted spelling `'f1'`
  (`quote_unbound_cell_shaped_names`, run only on text holding `_xlpm.` or typed in R1C1 mode). It evaluates to
  `#NAME?` (no LET declares it, no defined name matches), A1 output prints it quoted, and the R1C1 storage form, the
  export and the A1 text of R1C1 entry write it `_xlpm.f1` (`quoted_name_text`), which reads back quoted.
  `r1c1_formula_to_a1` no longer refuses such a formula. The review MINORs: `reads_back_as_cell`'s doc corrected,
  and `spelled_like_a1_cell` is the one helper both it and `prefix_a1_cell_shaped_variables` use.
- Eval: probes known failures 4226 -> 4217 (12 fixed, 3 new-case cells, 0 regressions). probes/t3897-r1c1-let-cell-names now passes all 39 cells (A8:C10, A13:C13 fixed, 12 cells); the new
  probes/t4040-r1c1-unbound-cell-names passes 39 of 42 (C14:C16 owned by T-4099).
