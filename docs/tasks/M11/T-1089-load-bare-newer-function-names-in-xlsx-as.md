---
id: T-1089
title: "Load bare newer function names in xlsx as #NAME? and export eta arguments as _xleta."
milestone: M11
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
Workbooks round-trip function names as Excel stores them: a newer function stored without its
`_xlfn.` prefix evaluates to `#NAME?` on import, and an eta-reduced function argument
(`GROUPBY(..., SUM)`) is exported as `_xleta.SUM`.

## Context
docs/excel-semantics/xlsx-storage.md, "Newer functions need `_xlfn.` in the file, eta arguments
`_xleta.`" (probes/catalog/function-name-forms A1-A6, function-name-forms-2 A1-A3; T-0040).
- Import: the parser strips `_xlfn.`/`_xlfn._xlws.` and then looks the name up, so a bare
  `PERCENTOF(1,2)` read from a file resolves to the function
  (crates/rustsheet-engine/src/expressions/parser/mod.rs, around lines 712-787). In Excel it is
  `#NAME?`. The same parser reads typed formulas, where the bare name is right, so the rule must
  apply only to formulas read from xlsx. That needs a parse mode or an import-time check against
  the list of names that need a prefix (`Function::to_xlsx_string`).
- Export: `Function::to_xlsx_string` writes `_xlfn.` names, but nothing writes `_xleta.` for a
  function passed by name. Excel reads such a bare name as `#NAME?`.
- `parse_named_call` (expressions/parser/operands.rs) maps a bare `SINGLE(` to `@` and a bare `ANCHORARRAY(` to `#` before
  the name lookup, in every mode (typed SINGLE is `@` in Excel, T-2938). The import rule must cover those two arms too: a
  bare `SINGLE(` or `ANCHORARRAY(` read from a file is likely `#NAME?` (probe it first).
Tests need xlsx inputs (an import test with a bare `_xlfn`-less newer name) and an export test.
- corpus-bench/1726 `July-KPIs!A1` `=IMAGE("https://...")` stored without `_xlfn.`: Excel `#NAME?`, engine `#BLOCKED!` (T-0427 run; the case's other 33 cells are T-1770).

## Acceptance criteria
- [x] Importing a workbook whose `<f>` is `PERCENTOF(1,2)` gives `#NAME?`; `_xlfn.PERCENTOF(1,2)`
  gives 0.5; typed formulas are unchanged.
- [ ] Exporting `=GROUPBY(A1:A2,B1:B2,SUM)` writes `_xleta.SUM`, and re-importing evaluates the same.
  Split to T-5011 (see Outcome).
- [x] `make check` green; reviewers approve. (`make check-fast` locally; CI runs `make check`.)

## Out of scope
- Implementing GROUPBY/PIVOTBY.

## Outcome
- Import: a new parser mode, `Parser::set_file_names`, is set only by the xlsx cell importer
  (rustsheet-xlsx import/worksheets.rs). In it, a call of a function whose `Function::to_xlsx_string` starts
  `_xlfn.`, written without a prefix and in any case, is no function (`needs_stored_prefix`,
  expressions/parser/operands.rs) but an unknown name, so `#NAME?`, as Excel loads it
  (probes/catalog/function-name-forms A1). The call keeps its name: the R1C1 storage marks it `_xlpm.`, as it does
  any name that reads as a function, so `to_bytes`/`from_bytes`, a save (written bare) and a reload all read it the same.
  Typed and API text and the harness's `Input::StoredFormula` (whose case builder adds the prefixes) are unchanged.
- Tests: rustsheet-xlsx/tests/bare_newer_function_names.rs (bare, prefixed and lower-case PERCENTOF; bare and
  prefixed SUM; bare and prefixed XLOOKUP; `_xlfn._xlws.SORT`; the internal and xlsx round trips; typed text).
  Two existing tests used bare newer names in files: table_part_names.rs's fixture now stores
  `_xlfn.FORMULATEXT`, and test.rs's openpyxl example expects `#NAME?` for its bare `CONCAT` (openpyxl's own docs
  say such formulas must be prefixed with `_xlfn.`).
- Split: the `_xleta.` export needs a typed bare function name to be read as a function passed by name, which the
  engine does not do (typed `=BYROW(A1:A2,SUM)` is `#VALUE!`, `=GROUPBY(..,SUM)` `#NAME?`) and which is unprobed.
  That is T-5011.
- Left as they are (xlsx-storage.md "Known gaps"): a bare `SINGLE(`/`ANCHORARRAY(` in a file (unprobed), a bare newer
  name in a defined name's body, and the legacy `@` the engine shows on the unknown call.
- Eval delta: corpus-bench 2 → 1 known failures (1726, the bare `=IMAGE(...)` Excel shows as `#NAME?`), and its
  round-trip line goes; 0 regressions. roundtrip-calipers pv_goldens/pv26_2_002 times out at 60 s on this machine
  with and without the change (2m12s vs 2m19s at a 300 s limit), so its baseline is untouched (not a CI gate suite).
