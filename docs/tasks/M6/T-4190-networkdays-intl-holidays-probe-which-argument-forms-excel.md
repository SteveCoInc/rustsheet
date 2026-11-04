---
id: T-4190
title: "NETWORKDAYS.INTL holidays: probe which argument forms Excel treats as a scalar value (the weekend-independent drop) beyond literals, arithmetic and SUM, and match it"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
NETWORKDAYS.INTL gives Excel's count for every common way of writing its holidays argument, not only the forms
probed so far. Excel splits holidays into two kinds. A scalar value is dropped only on Saturday or Sunday, whatever
the weekend argument says. A reference or an array is dropped on the weekend argument's own days. The engine decides
the kind from an allowlist of syntax-node kinds. This task replaces that allowlist with Excel's evidenced rule.

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 5941bb95). `make unowned-off ALL=1` lists 3 visibly-off cells
  with no open owner, all owned by deferred tasks. The functions-date lane has no ready opus task, so the rules the
  engine still guesses are the gap left in this lane. This one is recorded as UNVERIFIED in docs/excel-semantics/dates.md,
  "NETWORKDAYS.INTL: a scalar holiday ignores the weekend argument" (T-3537).
- Evidenced (excel-graph, T-3537): probes/networkdays-intl-holiday-forms A1:A22, networkdays-intl-holiday-weekend-masks,
  networkdays-intl-holiday-weekend-dedupe, networkdays-intl-early-holidays and networkdays-intl-serial-zero-holiday.
  These cells show the split. `NETWORKDAYS.INTL(45000,45100,13,45003)` (a Saturday under weekend 13, Tuesday only) is
  87, so the holiday is dropped. With `C1` = 45003 the same call is 86, as it is with `{45003}`. `45003+0` and
  `SUM(C1)` are scalar.
- Ours: `Holidays::scalar` is set in `holiday_serials`. A value that is not an array or range counts as scalar only
  when `is_scalar_node` accepts the argument's node (crates/rustsheet-engine/src/functions/date_and_time/holidays.rs).
  `is_scalar_node` accepts numbers, `+ - * / ^`, unary operators and every function except INDEX, OFFSET, INDIRECT,
  CHOOSE and IF. Every other node is treated as a reference: a string literal (`"45003"`), a boolean, a defined name,
  a LET variable, a LAMBDA parameter or call, `@C1`. Nothing in the evidence decides these. The excluded functions are
  a guess too: dates.md says only SUM, literals and `+` are probed. Other reference-returning functions are counted as
  scalar: XLOOKUP, IFS, SWITCH, IFERROR, LET returning a reference, TAKE/DROP of a range, TRIMRANGE, a `:` range
  between two INDEX calls.
- Design question for the PR: either Excel classifies by syntax, or by whether the evaluated argument is a reference
  (a value-kind rule that the engine's `CalcResult::Range` would give directly). The probe must tell the two apart.
  Include pairs that agree in value kind but differ in syntax: `LET(h,C1,h)` against `LET(h,45003,h)`,
  `IF(TRUE,C1)` against `IF(TRUE,45003)`, `CHOOSE(1,45003)` against `CHOOSE(1,C1)`. Also pairs that agree in syntax
  but differ in value kind: `XLOOKUP(1,{1},C1:C1)` against `XLOOKUP(1,{1},{45003})`, and a defined name bound to
  `=45003` against one bound to `=Sheet1!$C$1`.
- Also probe: `"45003"` as text, `--"45003"`, `(C1)`, `@C1`, `+C1`, `INDEX(C1:C2,1)` (a reference by syntax), `N(C1)`,
  `T(C1)&""` type cases, a LAMBDA call `LAMBDA(x,x)(C1)` and `LAMBDA(x,x)(45003)`, `IFERROR(C1,0)`, `SWITCH(1,1,C1)`.
  Use weekend 13 with holidays on a Saturday (45003) and on a Tuesday (45006) so each form gives 86 or 87 either way,
  as the existing probe does. Repeat a few of the forms for WORKDAY.INTL, which dates.md says has no split
  (`WORKDAY.INTL(45000,10,13,45003)` is 45012), so the fix does not leak into it.
- Related open tasks, not duplicated here: T-2864 (date argument kinds: empty text and errors in the date parameters,
  research), T-2591 (WORKDAY.INTL/NETWORKDAYS.INTL weekend TRUE against 2/3), T-1954 (WORKDAY lift controls).

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph, canary-verified, tag M6) holds the forms above under weekend 13,
      with the Saturday and the Tuesday holiday each, plus the WORKDAY.INTL controls.
- [x] dates.md "NETWORKDAYS.INTL: a scalar holiday ignores the weekend argument" states the rule that separates the two
      kinds, cites the new cells, and drops its UNVERIFIED line (or narrows it to the forms still unprobed).
- [x] The engine matches every new cell. `is_scalar_node`'s doc comment cites the evidence. If the rule is
      value-based, the node allowlist is deleted, not kept beside it.
- [x] probes/networkdays-intl-holiday-forms, -weekend-masks, -weekend-dedupe, -early-holidays and
      -serial-zero-holiday still pass. Eval known failures only shrink (`make eval-baseline`, 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- WORKDAY.INTL's own holiday rules beyond the controls (already evidenced), NETWORKDAYS without INTL (its weekend is
  always Saturday and Sunday, so the split cannot show), and the argument kinds T-2864 owns.

## Outcome
- Probes (excel-graph, 2026-11-04): `networkdays-intl-holiday-kinds` (M6, 32 forms with a Saturday and a Tuesday holiday
  under weekend 13, WORKDAY.INTL controls, and ISREF of the unary forms), `-kinds-let` (M7: LET, LAMBDA, TAKE, DROP) and
  `-kinds-names` (M8: defined names bound to `=45003` and to `=Sheet1!$C$1`). 121 cells.
- Rule: the split is by value kind. A holidays argument is a reference exactly when ISREF of it is TRUE. The syntax pairs
  separate it: `IF(TRUE,C1)` is a reference but `IF(TRUE,45003)` is not, and `LET(h,C1,h)` is a reference but
  `LET(h,45003,h)` is not. `+C1` is scalar and `@C1` is a reference, and ISREF agrees on both.
- Engine: `Holidays::scalar` is now `!array && !is_reference(arg)`, and the `is_scalar_node` allowlist is deleted. To make it
  work, `Model::is_reference` (ISREF, T's cell-text rule) gained `written_reference`: a unary plus among the written
  wrappers is no reference, and `@` of one cell is one. Before this, ISREF got `+C1` and `@C1` wrong.
- dates.md entry: VERIFIED, with the form list and the cited cells. references.md has a new entry,
  "ISREF: a unary plus makes a value".
- Eval delta: probes +3 passing cases (121 cells), 0 regressions. Before the fix, 26 of these cells were off. The listed
  existing networkdays-intl probes still pass.
