---
id: T-4871
title: "A written-empty search_mode of XLOOKUP and XMATCH and an empty rows or columns of EXPAND follow Excel (probe empty-as-omitted)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XLOOKUP and XMATCH with a written-empty `search_mode` (`XLOOKUP(1,A1:A2,B1:B2,0,0,)`, `XMATCH(2,A1:A2,,)`) and
EXPAND with a written-empty `rows` or `columns` (`EXPAND(A1:B1,3,,0)`, `EXPAND(A1:A2,,3)`) give what Excel gives,
decided by a small probe whose predictions are committed before the fill.

## Context
- Blind spot: the generator never writes an empty argument (`call_args` in tools/src/rustsheet_tools/gen/generator.py
  only drops trailing optionals), and no eval cell has an empty search_mode or an empty EXPAND size (gap triage
  2026-11-07, `make eval` on origin/main 3d991bd3d).
- Ours today (scratch eval, unverified): an empty argument evaluates to `CalcResult::EmptyArg`, which the mode and size
  readers cast to 0, an invalid value:
  - `XLOOKUP(1,{1,2},{3,4},0,0,)` and `XLOOKUP(2,{1,2},{3,4},,,)` `#VALUE!` (the same call without the trailing comma is
    3 / 4); `XMATCH(0,{1},1,)`, `XMATCH(2,{1,2},0,)` and `XMATCH(2,{1,2},,)` `#VALUE!` (`XMATCH(0,{1},1)` is 1).
  - `EXPAND({1,2},3,,0)`, `EXPAND({1;2},,3,0)`, `EXPAND({1,2},,,0)`, `EXPAND(5,,,0)` and `EXPAND(1,2,,"")` `#VALUE!`.
- Excel evidence that the dynamic-array functions read a written-empty argument as omitted, not as 0:
  - TAKE and DROP keep every row or column for an empty count (arrays.md "TAKE and DROP with an omitted count",
    probes/take-drop-omitted-rows F1:M6), while a blank cell there is 0 (`#CALC!`).
  - SORT's empty sort_order takes its default although 0 is invalid there (arrays.md "SORT: omitted sort_index /
    sort_order take their defaults", probes/sort-omitted-args H1:H45); `SORTBY(A1:A3,C1:C3,)` is 1 (same case).
  - `RANDARRAY(,,7,7)` is 7 (probes/t3631-randarray-invariant); `XMATCH(1,{1,2,3},)` is 1 (lookup.md "XMATCH with an
    omitted argument", probes/counta-empty-value D20).
  - Against: the legacy lookups read an empty argument as 0/FALSE (`VLOOKUP(..,)` exact), and Microsoft's EXPAND page
    says "If missing, rows will not be expanded". The probe decides; the prediction to commit is "empty = omitted"
    for XLOOKUP, XMATCH and EXPAND.
- Code: `xlookup_read` (functions/xlookup.rs: `args.get(4)` / `args.get(5)` default only when the node is absent, not
  for `Node::EmptyArgKind`; `xlookup_modes` casts the value), `fn_xmatch` (functions/lookup_and_reference/xmatch.rs,
  same pattern at `args.get(2)` / `args.get(3)`), `fn_expand` (functions/lookup_and_reference/expand.rs:
  `cast_number_or(values.get(1), 0.0, ..)` and `cast_number_ref` of `values.get(2)`). TAKE/DROP's evidenced handling
  is `None | Some(CalcResult::EmptyArg) => Ok(None)` in drop_take.rs: reuse that shape, no new helper per function.
  Keep the lifted forms working: an array `search_mode` or `rows` still lifts (probes/xlookup-array-lookup-value-args,
  probes/m7-argument-modes F7:H7).
- How it avoids fitting: each cell is either the default-mode result or `#VALUE!`; nothing numeric is tuned.

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph) with, over a small Data sheet: XLOOKUP with an empty
      if_not_found, match_mode and search_mode, one at a time and together, hit and miss; XLOOKUP with search_mode -1
      next to an empty match_mode; XMATCH with an empty match_mode and/or search_mode on an ascending and a descending
      vector; EXPAND with an empty rows, an empty columns and both, with and without pad_with, read through ROWS,
      COLUMNS and INDEX so nothing spills into another probe; and `LET(e,,XMATCH(2,A1:A3,0,e))` for the omitted-variable
      path.
- [x] The engine's predictions (empty = omitted for these three functions) are committed in a commit before the oracle
      fill.
- [x] After the fill every probe cell passes; lookup.md (XLOOKUP/XMATCH) and arrays.md (EXPAND) state the rule with
      its cells.
- [x] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- The legacy lookups (VLOOKUP, HLOOKUP, MATCH, LOOKUP, ADDRESS): their empty arguments are 0/FALSE today; file a
  follow-up probe only if this probe shows the rule is not limited to the dynamic-array functions.
- WRAPROWS/WRAPCOLS' empty wrap_count (`#NUM!` today), unless the same code path makes it free.

## Outcome
- **Finding:** XLOOKUP and XMATCH read a written-empty `match_mode` or `search_mode` as omitted, and so does a LET
  variable bound to nothing in those places. EXPAND reads a written-empty `rows` or `columns` as the array's own size.
  None of them reads the empty argument as 0.
- **Evidence:** probes/t4871-empty-mode-args, 31 cells, excel-graph, 2026-11-07. The predictions were committed in
  3a0ffa3eb before the fill, under two hypotheses. H_omitted is the engine's value of the same call with the empty
  argument written as its default. H_zero is the engine before this change. Excel matched H_omitted on 31/31 and
  H_zero on 9/31. Every cell is a default-mode result or an error, so nothing numeric was tuned.
- **Code:**
  - New `mode_or_default` in functions/lookup_and_reference/xmatch.rs, used by `fn_xmatch` and `xlookup_read`. An
    empty match_mode already worked, because its default is 0; search_mode, whose default is 1, did not.
  - `fn_expand` gets `CalcResult::EmptyArg` arms with the same shape as TAKE and DROP.
  - Unit test: test/test_empty_mode_args.rs.
- **Docs:** lookup.md "XLOOKUP and XMATCH read a written-empty mode as omitted (T-4871)" and arrays.md "EXPAND with a
  written-empty rows or columns keeps the array's size (T-4871)".
- **Not covered:** the legacy lookups. The probe tested only the dynamic-array functions, so this says nothing about
  them, and no follow-up is filed. WRAPROWS and WRAPCOLS are untouched; their path differs.
- **Eval:** one new case, 31/31.
