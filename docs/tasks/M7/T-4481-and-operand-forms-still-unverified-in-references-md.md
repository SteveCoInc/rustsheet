---
id: T-4481
title: "@ and # operand forms still UNVERIFIED in references.md: @(INDEX(..)), @(D1#), @(name), (@(A1 B1))#, a typed (1/0)#, @TRUE and two error operands of :, probe and match"
milestone: M7
lane: engine
status: done
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
- [x] probes/t4481-at-spill-operand-edges (excel-graph, canary-verified) holds every loadable form in Context, with
      FORMULATEXT and value cells. Each refused form is listed with its load error in the Outcome and in
      references.md.
- [x] Every cell of the case passes, or each one that does not is recorded in references.md and owned by a filed task
      named in the Outcome.
- [x] references.md: the "not probed", "still inferred" and "UNVERIFIED" clauses quoted in Context are replaced by the
      evidenced behavior, with cells cited. The parser accepts or refuses each refused form as Excel does, and a
      parser unit test covers each such form.
- [x] No regressions in any baseline; `make check` green; reviewers approve.

## Out of scope
- Legacy (no `cm`) formulas' automatic `@` and the implicit intersection of a defined name's body (T-3856, T-4273).
- Move and paste text (`to_string_moved`).

## Outcome
Probed; the engine already matches every cell, so only a parser test and the docs changed.
- probes/t4481-at-spill-operand-edges (excel-graph, filled by `rustsheet-oracle fill`, which canary-verifies every
  workbook): the 17 loadable forms in Context, each with FORMULATEXT and ERROR.TYPE (51 cells, plus D1's spill).
  `docs/research/t4481/gen_probe.py` writes it, and the engine's predictions (`docs/research/t4481/predictions.tsv`)
  were committed before the fill. 51/51 match.
- Refused forms: `=(1/0)#` and `=(NA())#`, stored as `_xlfn.ANCHORARRAY((1/0))` and `_xlfn.ANCHORARRAY((NA()))`. Excel
  for the web refuses each one (createSession 501 `FileCorruptTryRepair`, one scratch workbook each), while
  `=ANCHORARRAY((A1:A2))` loads in the same shape. The parser already refused `(1/0)#`.
  `range_and_anchorarray_refuse_value_operands` now also covers `(NA())#` and both stored spellings. spec2xlsx copies
  a `#` written after a parenthesis into the file as is, which Excel also refuses, so the case writes these forms as
  `ANCHORARRAY((…))`.
- Findings (references.md):
  - `@(INDEX(…))`, `@(D1#)`, `@(name)` and `(@name)` keep their written parentheses.
  - `@TRUE` and `@(TRUE)` are TRUE, not `#VALUE!`.
  - An `@` error passes through `#`.
  - With two error operands, `:` returns the left one.
  - The "not probed", "still inferred" and two "UNVERIFIED" clauses are replaced with cited cells.
- Eval delta: +1 passing probe case (54 cells; eval/baseline/probes.txt records it), 0 regressions.
