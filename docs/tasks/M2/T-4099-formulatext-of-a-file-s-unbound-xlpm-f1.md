---
id: T-4099
title: "FORMULATEXT of a file's unbound _xlpm.f1 shows the prefix: =_xlpm.f1"
milestone: M2
lane: engine
status: todo
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
- [ ] probes/t4040-r1c1-unbound-cell-names A14:C16 match Excel, and A7:C13 still do.
- [ ] docs/excel-semantics/syntax.md "Short R1C1 forms typed in R1C1 mode" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed R1C1 entry of such a name (T-4040).
