---
id: T-4481
title: "@ and # operand forms still UNVERIFIED in references.md: @(INDEX(..)), @(D1#), @(name), (@(A1 B1))#, a typed (1/0)#, @TRUE and two error operands of :, probe and match"
milestone: M7
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The implicit-intersection (`@`) and spill (`#`) operand forms that docs/excel-semantics/references.md still lists
as unprobed or inferred have Excel's FORMULATEXT, value and load behavior on excel-graph. The engine matches each of
them, or the gap is recorded and owned.

## Context
- references.md "FORMULATEXT keeps the parentheses of an `@` operand; ANCHORARRAY is `#`; `:` passes an error operand
  on": "Excel keeps the parentheses the user wrote around the operand of `@`, for every operand form probed
  (intersection, range, single cell, nested `@`; `@(INDEX(…))`, `@(D1#)` and `@(name)` are not probed)". The same
  entry: "The `#VALUE!` for the `@` of a non-number value is probed for `@"a"` and `@SUM(A1)` (T-1240); `@TRUE` is
  still inferred", and "Still UNVERIFIED: a value operand of a typed `#` (`(1/0)#`)".
- references.md, the intersection entry's ANCHORARRAY bullet: "UNVERIFIED: the engine passes any `@` error through `#`
  (`=(@(A1 B1))#` gives `#NULL!`), and gives the left error when both operands of `:` are errors; only the `#VALUE!` and
  right-operand cases are probed."
- `@`, the spill operator and their interaction are M7 scope (docs/milestones.md: "implicit intersection (`@`)").
  None of these forms appears in eval/suites, and T-1104, T-1111, T-1216, T-1240 and T-1246 (all done) left them out.
- Ours: `Parser::parse_implicit` and `spill_operand` (crates/rustsheet-engine/src/expressions/parser/
  reference_operators.rs) parse and print them, `is_reference_operand` decides which operands `:` and `#` accept, and
  the stringify module keeps a written `Node::Group` around an `@` operand (T-1343).
- Probe (excel-probe skill, excel-graph, tag M7). D1 holds `=SEQUENCE(3)`, F1:F3 = 1, 2, 3, the name `rg` =
  `=Sheet1!$F$1:$F$3` and `one` = `=Sheet1!$F$2`. Each formula sits in its own row, with FORMULATEXT beside it, and with
  ERROR.TYPE where the value is an error:
  - `=@(INDEX(F1:F3,2))`, `=@INDEX(F1:F3,2)`, `=@(D1#)`, `=@D1#` as a control (evidenced `#VALUE!`), `=@(rg)`, `=@rg`,
    `=@(one)`, `=(@rg)`, the last with the formula on row 2 so that the intersection picks F2;
  - `=@TRUE`, `=@(TRUE)`, `=@FALSE+1`;
  - `=(@(A1 B1))#`, `=(@(F1:F3 F2))#`;
  - `=(@(A1 B1)):INDEX(F1:F3,5)` (two error operands of `:`), `=INDEX(F1:F3,5):(@(A1 B1))`;
  - `=(1/0)#` and `=(NA())#`, each in its own scratch workbook first. A workbook Excel for the web refuses
    (`FileCorruptTryRepair`) is evidence of the refusal, as T-1216 recorded; keep only the loadable forms in the case.
- Write the engine's values and FORMULATEXT as predictions before the fill (docs/research/t4481/).

## Acceptance criteria
- [ ] probes/t4481-at-spill-operand-edges (excel-graph, canary-verified) holds every loadable form in Context, with
      FORMULATEXT and value cells. Each refused form is listed with its load error in the Outcome and in
      references.md.
- [ ] Every cell of the case passes, or each one that does not is recorded in references.md and owned by a filed task
      named in the Outcome.
- [ ] references.md: the "not probed", "still inferred" and "UNVERIFIED" clauses quoted in Context are replaced by the
      evidenced behavior, with cells cited. The parser accepts or refuses each refused form as Excel does, and a
      parser unit test covers each such form.
- [ ] No regressions in any baseline; `make check` green; reviewers approve.

## Out of scope
- Legacy (no `cm`) formulas' automatic `@` and the implicit intersection of a defined name's body (T-3856, T-4273).
- Move and paste text (`to_string_moved`).
