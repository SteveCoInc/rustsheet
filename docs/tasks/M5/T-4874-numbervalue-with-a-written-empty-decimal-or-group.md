---
id: T-4874
title: "NUMBERVALUE with a written-empty decimal or group separator follows Excel (probe empty-as-omitted)"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
NUMBERVALUE with a written-empty `decimal_separator` or `group_separator` (`NUMBERVALUE("1.5",,",")`,
`NUMBERVALUE("1,234.5",".",)`) gives what Excel gives, decided by a small probe whose predictions are committed before
the fill.

## Context
- Blind spot: the generator never writes an empty argument (`call_args` in tools/src/rustsheet_tools/gen/generator.py
  only drops trailing optionals), and no eval cell calls NUMBERVALUE with one (gap triage 2026-11-07, `make eval` on
  origin/main 3d991bd3d). The two NUMBERVALUE cells with `,,` in probes are inside the text (`NUMBERVALUE(",,1")`).
- Ours today (scratch eval, unverified): every written-empty separator is `#VALUE!`: `NUMBERVALUE("1.5",,",")`,
  `NUMBERVALUE("1,234.5",".",)`, `NUMBERVALUE("2,5",,)`, `NUMBERVALUE("1:",,"0")` and `NUMBERVALUE("1:","F",)`, while
  the same calls with the argument left out are 1.5, 1234.5, 25 and 0.0416666666666667.
- Excel evidence around it: an empty-text separator is `#VALUE!` (`NUMBERVALUE(2,"")`, `NUMBERVALUE("abc","","")`:
  gen-m5/c8e15c49524fca07 Calc!A25, gen-m5/f0b54c517d756328 Calc!A175), and so is a blank-cell one
  (`NUMBERVALUE(0,Data!H1)`, gen-m5/c8e15c49524fca07 Calc!A97). The question is whether a written-empty argument is that
  empty value or an omitted one. Functions added since Excel 2010 read it as omitted where probed: TAKE/DROP
  (arrays.md "TAKE and DROP with an omitted count"), SORT's sort_order (arrays.md "SORT: omitted sort_index /
  sort_order take their defaults"), and in this lane TEXTBEFORE/TEXTAFTER/TEXTSPLIT (`TEXTAFTER("a","İ",,1)` and
  `TEXTSPLIT("Σx","ς",,,1)` in probes, passing). NUMBERVALUE is a 2013 function, so the prediction to commit is
  "empty = omitted" (the locale's `.` and `,`).
- Code: `fn_numbervalue` (functions/text/string_format.rs). Keep a blank cell and `""` as `#VALUE!` (the cells above);
  only `Node::EmptyArgKind` (and a LET/LAMBDA variable bound to nothing, `is_omitted_variable`, as `complex_pair_of`
  in engineering/complex/mod.rs reads it) changes, if the probe says so.
- How it avoids fitting: each cell is the default-separator parse or `#VALUE!`; nothing numeric is tuned.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) calls NUMBERVALUE with an empty decimal_separator, an empty
      group_separator and both, on "1.5", "1,234.5", "1.234,5", "2,5", "50%" and "1:30", plus
      `LET(e,,NUMBERVALUE("1.5",e))` and, as controls, the blank-cell and `""` forms.
- [ ] The engine's predictions (empty = omitted) are committed in a commit before the oracle fill.
- [ ] After the fill every probe cell passes; text.md (or coercion.md beside the NUMBERVALUE entries) states the rule
      with its cells.
- [ ] No new F line in any eval/baseline file (the gen-m5 cells above keep passing); `make check` green; reviewers
      approve.

## Out of scope
- The legacy text functions (SUBSTITUTE, FIND, SEARCH, LEFT, RIGHT, MID, REPT, FIXED, DOLLAR), which read an empty
  argument as 0 today; a follow-up probe only if this one shows the rule reaches them.
