---
id: T-4099
title: "FORMULATEXT of a file's unbound _xlpm.f1 shows the prefix: =_xlpm.f1"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A file's formula that stores an unbound name spelled like an A1 cell with the `_xlpm.` prefix (`=_xlpm.f1`,
`=_xlpm.AB12*2`, `=SUM(_xlpm.xy1,2)`) shows that text in FORMULATEXT, as Excel does, while it stays `#NAME?`.

## Context
- Found by T-4040. probes/t4040-r1c1-unbound-cell-names (excel-graph, 2026-11-04) A14:C16: the three stored formulas
  are `#NAME?` (ours matches) and FORMULATEXT is `=_xlpm.f1`, `=_xlpm.AB12*2`, `=SUM(_xlpm.xy1,2)`: Excel keeps the
  prefix of a file's unbound name. Ours prints the quoted spelling typed R1C1 entry gets, `='f1'`
  (`stringify::bound_variables::quote_unbound_cell_shaped_names`, run by `Parser::parse` on any text with `_xlpm.`).
- The two need different nodes (or a flag on `NamedVariableKind`) that the R1C1 storage form keeps apart: today both
  store as `_xlpm.f1` (`quoted_name_text`). Unprobed: whether a file's unbound `_xlpm.x` (a name that is no cell)
  shows its prefix too, and what Excel writes to a file for a typed R1C1 `=f1`; probe both first.

## Acceptance criteria
- [x] probes/t4040-r1c1-unbound-cell-names A14:C16 match Excel, and A7:C13 still do.
- [x] docs/excel-semantics/syntax.md "Short R1C1 forms typed in R1C1 mode" updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Typed R1C1 entry of such a name (T-4040).

## Outcome
- Probed first: probes/t4099-file-xlpm-unbound-names (excel-graph, 2026-11-04, 39 cells). Excel keeps the `_xlpm.`
  prefix on any unbound name written with it, not only a cell-shaped one, and typed in A1 or R1C1 mode as well as in a
  file: `=_xlpm.abc`, `=_xlpm.R1C1`, `=_xlpm.f1(2)`, `=LET(x,1,x+_xlpm.f1)` all show the prefix and are `#NAME?`.
  Only a bare `f1` typed in R1C1 mode is quoted. Recorded in syntax.md "Short R1C1 forms typed in R1C1 mode".
- Engine: the parser keeps the prefix on every use (`Parser::written_lambda_name`) and drops it from the bound ones
  at the end of the parse (`call_names::settle_lambda_prefixes`). A declaration drops it in `bind_bare_name`;
  `walk_names` matches a prefixed use against its bare declaration.
- Two internal texts tell the prefix apart from their own marks:
  - The R1C1 storage form already used one `_xlpm.` for a name that would read back as something else. It writes a
    written-prefix name with two (`variable_text`, `callee_text`), and reads one prefix as its own mark, two as the
    name's.
  - The A1 text of a formula typed in R1C1 mode (`r1c1_formula_to_a1`) writes the quoted `'f1'` as `_xlpm._xlpm.f1`.
    One prefix typed in A1 is now the prefixed name, and the A1 parser reads the doubled one back quoted.
- Unprobed (recorded): what Excel writes to a file for a typed R1C1 `=f1`. Ours exports `_xlpm.f1`, as before.
- Tests: new `an_unbound_prefixed_name_keeps_its_prefix` (file and typed, plus a reload from storage);
  `let_and_lambda_names_spelled_like_a1_cells_keep_their_prefix` now expects the doubled A1 text.
- Eval delta: probes fixed 3 (t4040-r1c1-unbound-cell-names C14:C16), +1 case (t4099, 39 cells, all pass),
  0 regressions.
