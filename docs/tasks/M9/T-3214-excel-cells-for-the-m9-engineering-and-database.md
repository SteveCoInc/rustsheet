---
id: T-3214
title: "Excel cells for the M9 engineering and database argument kinds no suite covers (119 parameter/kind pairs in 58 functions, mostly empty text, errors and a numeric zero)"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every value parameter of the 46 M9 engineering functions and the 12 database functions listed below has an
Excel-backed cell of each argument kind that report.json `argument_coverage` expects. Phase 1's exit needs this,
because decision 0094 asks for coverage "across the argument modes its catalog signature lists". Each new cell that
differs from Excel gets an owner.

## Context
Gap triage 2026-10-30 (`make eval` on main 8f47bb850; decision 0031 evidence gaps). T-3184 (financial) and
T-3185 (statistical) cover the other M9 argument-kind gaps. T-3184's Out of scope leaves out "the engineering and
database argument kinds", and no other task covers them. The kinds come from
crates/rustsheet-eval/src/argument_coverage.rs `classify_value`. Here, zero means a number equal to 0, not the text
"0". M9 engineering lacks 106 parameter/kind pairs in 46 functions (empty_text 42, error 40, zero 19, boolean 2,
blank 2, numeric_text 1). M9 database lacks 13 pairs in 12 functions (error 12, empty_text 1).

Missing kinds per parameter (report.json `argument_coverage`, catalog `category: engineering` or `database`,
`milestone: M9`, `covered < expected`):
- An empty text in the first parameter, and empty text plus an error in the second: BESSELI, BESSELJ, BESSELK and
  BESSELY (x, then n). Also BITAND, BITOR and BITXOR (number1, then number2), BITLSHIFT and BITRSHIFT (number,
  then shift_amount), and DELTA and GESTEP (number1/number, then number2/step).
- An error, and empty text in places: BIN2HEX.number (error); BIN2HEX, BIN2OCT, HEX2BIN, HEX2OCT, OCT2BIN and
  OCT2HEX .places (empty text, error); OCT2BIN, OCT2DEC and OCT2HEX .number (error).
- DEC2BIN, DEC2HEX and DEC2OCT .number lack empty text and an error. DEC2BIN.places lacks empty text, a boolean
  and an error. DEC2HEX.places lacks empty text, an error and a blank. DEC2OCT.places lacks empty text and an error.
- HEX2BIN, HEX2DEC and HEX2OCT .number lack a numeric zero. HEX2BIN and HEX2OCT also lack an error.
- A numeric zero in inumber: IMCOS, IMCOSH, IMCOT, IMCSC, IMCSCH, IMEXP, IMLN, IMLOG10, IMREAL, IMSEC, IMSECH,
  IMSIN, IMSINH. IMAGINARY.inumber lacks a zero and a blank. IMDIV.inumber2 and IMPOWER.inumber lack a zero and an
  error. The generator's `complex_text` pool (tools/src/rustsheet_tools/gen/pools.py) has the text "0" but no
  number 0.
- An error: IMDIV.inumber1, IMSUB.inumber1 and .inumber2, IMPRODUCT.inumber (array mode), COMPLEX.suffix,
  CONVERT.number. IMPOWER.number lacks empty text and an error. COMPLEX.real_num lacks empty text; COMPLEX.i_num
  lacks numeric text, empty text and an error.
- ERF.lower_limit, ERF.PRECISE.x, ERFC.x and ERFC.PRECISE.x lack empty text. ERF.upper_limit lacks empty text, a
  boolean and an error.
- Database: an error in `field` for DAVERAGE, DCOUNT, DCOUNTA, DGET, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM,
  DVAR, DVARP. DCOUNTA.field also lacks empty text.

How to write it: the T-2861 style (excel-probe skill, excel-graph). Use one probe case per category,
probes/m9-engineering-argument-kinds and probes/m9-database-argument-kinds, tag M9. Give each kind in two forms
where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `0`, `TRUE`, `"2"`) and a cell holding it (a formula cell
`=""` or `=NA()`, a typed `0` or `TRUE`, an empty cell). Wrap every error-valued result in `ERROR.TYPE` too, so the
error kind is scored. Put the second-parameter errors after a valid first argument, and also after an error first
argument, which records the error precedence. The database cells use the `Db` sheet layout of
tools/src/rustsheet_tools/gen/database.py, with a valid criteria range, so only `field` varies.

Known Excel behavior to cite where it applies: docs/excel-semantics/engineering.md (base conversions, places, the
complex-number text rules, BITAND/BITLSHIFT domains) and docs/excel-semantics/coercion.md. The database rules are in
docs/excel-semantics/statistics.md "Database function argument modes" (T-1390) and criteria.md. T-2334 verified
IMCSC("0"), IMLOG2("0") and IMCOT("0") as #NUM! with the text "0" only; the number 0 is unprobed.

## Acceptance criteria
- [x] The two probe cases are committed with oracle values (excel-graph). After `make eval`, report.json
      `argument_coverage` shows `covered == expected` for every parameter listed above. If Excel refuses a workbook
      holding a kind, the kind is exempted in the catalog with its reason instead.
- [x] Each new cell where ours differs from Excel is handled in one of two ways. A one-line coercion fix in the
      function's own module is fixed in this PR. Anything else is listed by case and cell in a functions-engineering
      or functions-criteria task filed with `make new-task`, with `campaign: semantics` and a
      `## Visible-parity stage` section for wrong types and errors. The new known failures are recorded with
      `make eval-baseline`, and `make unowned-off ALL=1` lists none of them.
- [x] Each new Excel rule is in docs/excel-semantics/engineering.md, statistics.md (database), criteria.md or
      coercion.md, citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the generator (T-3184 decides whether a coverage-directed `--focus` mode replaces hand-written probes;
  if it lands first, using it here is fine). Committed gen-m9 cases stay unchanged.
- The financial (T-3184) and statistical (T-3185) argument kinds.
- Fixing the last-digit residue of new numeric cells: file it as tasks (`defer: phase-1b` when within 15 digits).

## Outcome
- On main 5f73c1bd the gap had shrunk from the triage's 119 pairs to 31 (later suites covered the rest): the 12
  database `field` errors, ERF.PRECISE/ERFC/ERFC.PRECISE empty text, HEX2DEC and 13 IM* numeric zeros, IMAGINARY
  blank, IMPRODUCT and OCT2DEC errors.
- Two probe cases, probes/m9-engineering-argument-kinds (39 cells) and probes/m9-database-argument-kinds (36
  cells), M9, excel-graph 2026-11-04: a valid base call with one argument swapped for the kind, as a literal and as a
  cell holding it (Data!A1 blank, A2 `=""`, A3 `=NA()`, A4 0). The database case uses the gen `Db` table with
  valid criteria. Error results are scored as errors already, so no `ERROR.TYPE` wrappers were added; the
  first-argument-error precedence cells were not needed, since no open pair was a second-parameter error of a
  two-argument engineering function, and a database given as an error literal risks a refused workbook.
- After `make eval`, report.json `argument_coverage` has `covered == expected` for every parameter listed in
  Context; no catalog exemption was needed.
- Engineering: all 39 match. Database: the literal errors propagate and match; `field` as a reference to an error
  cell is `#VALUE!` in Excel and `#N/A` in ours (Calc!A3, A6, A9, A12, A15, A18, A21, A24, A27, A30, A33, A36). That
  is not a one-line fix (the field arrives dereferenced), so it is owned by T-4247 (functions-criteria, campaign
  semantics) and baselined.
- Rules: engineering.md "M9 engineering argument kinds (T-3214)"; statistics.md "Database function argument modes"
  (the error-field rule).
- Eval delta: +75 checked cells, 63 passing, 12 new known failures (owned by T-4247); 0 regressions.
