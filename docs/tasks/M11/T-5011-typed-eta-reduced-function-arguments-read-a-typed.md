---
id: T-5011
title: "Typed eta-reduced function arguments: read a typed bare GROUPBY(..,SUM) as _xleta.SUM and export it so (T-1089 split)"
milestone: M11
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1089]
---

## Goal
A function passed by name in typed text (`=GROUPBY(B1:B2,A1:A2,SUM)`, `=BYROW(A1:A2,SUM)`) works as
Excel's typed entry does, and the xlsx export stores it `_xleta.SUM`, so a re-import evaluates the same.

## Context
- Split from T-1089 (its second acceptance criterion): the export has nothing to write `_xleta.` for, since the
  engine reads a typed bare `SUM` argument as an unknown name. On main, typed `=BYROW(A1:A2,SUM)` is `#VALUE!`,
  `=GROUPBY(B1:B2,A1:A2,SUM)` and `=LET(f,SUM,f(1,2))` `#NAME?` (checked with `set_typed_user_input`, T-1089).
- Stored form: docs/excel-semantics/xlsx-storage.md "Newer functions need `_xlfn.` in the file, eta arguments
  `_xleta.`" (probes/catalog/function-name-forms A3-A4); behaviour of `_xleta.` names: arrays.md "Functions passed by
  name" (`Model::unbound_name`, functions/logical/lambda.rs).
- Unprobed, so probe first (excel-probe skill, cells typed through Graph): which typed positions take a bare name as a
  function (helper LAMBDA arguments only, or any value, as `=LET(f,SUM,f(1,2))` and `=SUM`), how FORMULATEXT shows it
  (`SUM` or `_xleta.SUM`), and a bare name that is also a defined name or a LET name.
- Code: the parser's unbound-name node for a typed bare function name (expressions/parser/operands.rs), the
  stringifier's export arm (stringify.rs), `Function::to_xlsx_string` for the `_xleta._xlfn.` form of a newer
  function.

## Acceptance criteria
- [ ] A probe case with predictions committed before the fill pins the typed readings above.
- [ ] Typed `=GROUPBY(..,SUM)` evaluates as `_xleta.SUM` does, exporting writes `_xleta.SUM`, and re-importing
      evaluates the same (an xlsx round-trip test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A bare name read from a file (T-1089: an unknown name, `#NAME?`).
