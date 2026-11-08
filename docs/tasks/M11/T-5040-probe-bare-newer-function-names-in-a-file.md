---
id: T-5040
title: "Probe bare newer function names in a file across prefix families, lower case and the _xlws.-only form (T-1089 review follow-up)"
milestone: M11
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1089]
---

## Goal
Oracle evidence beyond PERCENTOF and IMAGE for how Excel loads a newer function stored without its `_xlfn.` prefix,
so the importer's rule (T-1089: every function whose `to_xlsx_string` starts `_xlfn.` is `#NAME?` bare) is confirmed
or corrected per family.

## Context
- T-1089 review (PR #4022, excel-fidelity MINORs). Today the rule rests on probes/catalog/function-name-forms A1
  (PERCENTOF) and corpus-bench/1726 (IMAGE). The prefix list has been wrong before (GAMMALN and ISO.CEILING, T-0417).
- docs/excel-semantics/xlsx-storage.md "Newer functions need `_xlfn.` in the file", Known gaps.
- Code: `needs_stored_prefix` (crates/rustsheet-engine/src/expressions/parser/operands.rs); tests
  crates/rustsheet-xlsx/tests/bare_newer_function_names.rs and test.rs `test_pyopenxl_example` (bare CONCAT).
- The case builder keeps a cell bare when the sheet lists it in `bare_function_names` (spec2xlsx.py). The harness
  reads case text as typed, so the cells are evidence and unscored, as function-name-forms A1 is.

## Acceptance criteria
- [ ] A probe case (predictions committed before the fill), each cell bare via `bare_function_names` and read through
      `IFERROR(ERROR.TYPE(..),0)`, covers: an Excel 2010 function (STDEV.S or NORM.DIST), CONCAT, XLOOKUP, a
      `_xlfn._xlws.` function stored bare (SORT) and stored `_xlws.SORT` without `_xlfn.`, a lower-case bare name
      (`percentof`), and a bare newer name inside a defined name's body.
- [ ] xlsx-storage.md records the results; any mismatch with the engine gets a task (or is fixed here if size S).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional-format and data-validation formulas (note what is seen, do not implement).
