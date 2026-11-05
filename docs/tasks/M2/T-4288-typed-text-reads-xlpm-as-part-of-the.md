---
id: T-4288
title: "Typed text reads _xlpm. as part of the name, and a formula typed in R1C1 mode carries its quoted name on the node, not in A1 text"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A formula typed in A1 or R1C1 mode reads `_xlpm.` as part of a name, as Excel does, and the quoted name `'f1'` of a
formula typed in R1C1 mode reaches the typed entry without an A1 spelling a user could type.

## Context
- From T-4224 (probes/t4224-doubled-xlpm-prefix, excel-graph): typed, `_xlpm.` is no mark. `=_xlpm._xlpm.f1` and
  `=_xlpm._xlpm.abc` show as typed (C3:C6, ours `='f1'` and `=_xlpm.abc`); `=LET(x,2,_xlpm.x*3)` and
  `=LET(x,2,_xlpm._xlpm.x*3)` are `#NAME?` (A9, A11, A14, C9, C11, C14; ours 6, `=LET(x,2,x*3)`); and
  `=LET(_xlpm.x,1,_xlpm.x+1)` is refused in either mode (A10, A12: B10, B12, C10, C12; ours accepts it).
  docs/excel-semantics/syntax.md has the rule.
- T-4224 made a file's (untyped A1) and the storage form's `_xlpm.` exactly one mark; the typed rows of
  `Parser::written_lambda_name` still collapse prefixes, because the R1C1 typed entry (`r1c1_formula_to_a1`) hands the
  typed A1 entry text that spells a bound cell-shaped name `_xlpm.f1` and the quoted `'f1'` `_xlpm._xlpm.f1`
  (`to_english_string_typed_from_r1c1`). Both spellings are a user's text too, so the R1C1 entry needs a path that
  keeps the parsed node (a typed-R1C1 entry on `Model`, used by `rustsheet-eval`'s `r1c1_entry`) or a node flag.
- `walk_names`' `declared` falls back to the name without one prefix; typed names would need an exact match.

## Acceptance criteria
- [ ] probes/t4224-doubled-xlpm-prefix S!A3:C6, S!A9:C12 and S!A14:C14 pass; t4040, t4099, t3897 and t3212 still pass.
- [ ] No A1 or R1C1 text a user can type reads as the quoted name; `r1c1_formula_to_a1` stays correct for its
      callers or is replaced.
- [ ] `make check` green; reviewers approve.

## Out of scope
- What Excel writes to a file for these typed formulas (unprobed).
