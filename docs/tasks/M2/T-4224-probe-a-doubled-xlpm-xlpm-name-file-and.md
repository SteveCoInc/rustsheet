---
id: T-4224
title: "Probe a doubled _xlpm._xlpm. name (file and typed) and a mixed LET(x,2,_xlpm.x*3), then mark the R1C1-typed quoted name on the node, not in its text"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence decides what a doubled `_xlpm._xlpm.` name shows, from a file and typed, and the quoted name of a
formula typed in R1C1 mode no longer rides on that spelling.

## Context
- From T-4099's review (PR #3190). The parser reads two `_xlpm.` prefixes as internal marks:
  - in A1 text, the quoted `'f1'` of `r1c1_formula_to_a1`;
  - in the R1C1 storage form, a name written with its prefix (`Parser::written_lambda_name`).
- A file or a user can write `=_xlpm._xlpm.f1` or `=_xlpm._xlpm.abc` too. Ours then shows `='f1'` or `=_xlpm.abc`;
  Excel is unprobed.
- Also unprobed, and covered only by a unit test: a bare declaration with a prefixed use, `=LET(x,2,_xlpm.x*3)`
  (ours `=LET(x,2,x*3)`, 6), and `=LET(_xlpm.x,1,_xlpm.x+1)`.
- A typed marker for the quoted name (a node flag, or a typed-entry API that takes the parsed node) would keep the
  internal texts apart from input.

## Acceptance criteria
- [x] A probe answers the doubled-prefix cells (file and typed in A1 and R1C1 mode) and the mixed LET; syntax.md
      records them.
- [ ] (split: the file half here, the typed half and the mark to T-4288) Ours matches them, and the quoted name's mark cannot clash with a file's or a user's text.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed R1C1 entry of other names (T-4040).

## Outcome
- Probe probes/t4224-doubled-xlpm-prefix (excel-graph, 40 cells): in a file exactly one `_xlpm.` is the mark and the
  rest is the name (a doubled prefix is an undefined name shown as written; `_xlpm._xlpm.x` declares `_xlpm.x`);
  typed, `_xlpm.` is part of the name (`=LET(x,2,_xlpm.x*3)` is `#NAME?`, a declaration with it is refused).
  syntax.md records both, and the unprobed neighbours.
- Ours: a file's and the storage form's `_xlpm.` is now one mark (`Parser::written_lambda_name`, `bind_bare_name`,
  `call_names::settle_lambda_prefixes`, `walk_names`' `declared`); the builder (`spec2xlsx.prefix_bound_names`) leaves
  a name already written `_xlpm.x` as the file's text, as the engine reads it (every existing case builds the same
  workbook). The case passes 26 of 40 cells (21 before); the 14 left are the typed rows, which need the typed reading
  and the R1C1 typed entry's quoted name off its A1 spelling: split to T-4288 (size M), owner of those cells.
- Eval: +5 cells (S!C1, S!C2, S!C8, S!A13, S!C13), no regressions.
