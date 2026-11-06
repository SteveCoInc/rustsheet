---
id: T-1856
title: "Refuse LET names and defined names Excel refuses at load: Turkish dotted İ and circled letters such as Ⓐ"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine refuses (as Excel does) a LET name or defined name that Excel refuses at load, such as the Turkish dotted `İ` and the circled letter `Ⓐ`, for formulas entered through the API as well as xlsx.

## Context
T-1431 (docs/excel-semantics/references.md, T-1387 paragraph) found that Excel for the web refuses at load (createSession 501 FileCorruptTryRepair) a single-cell workbook whose LET names `İ` or `Ⓐ` in any spelling of the use; the engine accepts them (`is_valid_a1_identifier` in crates/rustsheet-engine/src/expressions/utils/mod.rs uses `char::is_alphabetic`, which accepts `Ⓐ` through Other_Alphabetic; `İ` lowercases to two code points). T-1387 set the precedent for refused forms (duplicate LET names give `#ERROR!`). Do not guess the rule: probe `rustsheet-oracle recalc -b excel-graph` single-cell workbooks for which characters Excel accepts as the first and later characters of a name (LET, LAMBDA, defined name) and fit one rule; check formula entry vs. xlsx load.

## Acceptance criteria
- [x] Refusal probes recorded in references.md; the engine returns `#ERROR!` (or the repo's refused-name error) for the refused names; unit tests; no change to the accepted forms in probes/let-name-non-ascii*.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probes: 52 single-cell workbooks on excel-graph (2026-11-06; a 53rd, a defined `नाम`, failed on an oracle-side name
  collision, 409 nameAlreadyExists, and is not counted), recorded in references.md "Characters of a LET or LAMBDA
  name". A refused workbook has no oracle cells, so they cannot be committed as a case (T-1387's precedent). Rule:
  Excel refuses a LET or LAMBDA binding a name that holds a mark (`\p{M}`: `नाम`'s vowel sign, U+0345, U+0301) or an
  other symbol (`\p{So}`: `Ⓐ`, `ⓐ`, U+1F130), or whose lowercase does (`İ` → `i` + U+0307). Other letters (Lt, Lm,
  Lo, Nl, ligatures, Hebrew, Arabic) and later digits (`a²`, `a①`, `a٣`) load.
- Engine: `is_refused_variable_name` (functions/logical/let.rs) gives `#ERROR!` for such a LET name and, through
  `has_refused_parameters` (was `has_duplicate_parameter`), for such a LAMBDA parameter. One fix on the way:
  `is_name_text` checked the uppercase of a name, so `LET(ǰ,1,ǰ+1)` and `LET(ΐ,1,ΐ+1)` (letters whose uppercase holds
  combining marks) were `#ERROR!` where Excel gives 2; it now checks the characters as typed. Unit test
  `let_and_lambda_names_with_marks_or_symbol_letters_are_refused` (19 refused forms, 17 accepted).
- Defined names follow other rules: a defined `İ` works, and definitions `Ⓐ`/`aⒶ` load but their uses are `#NAME?`;
  a sheet formula `=Ⓐ+1` is refused at load. Filed as T-4446 with the evidence (the task's "defined name" half: the
  refusals there are not at the definition).
- Formula entry vs xlsx load: the oracle only loads xlsx; the engine applies the rule to both, as for T-1387's
  duplicate names.
- Eval delta: 0 fixed, 0 regressions (no committed case holds a refused or `ǰ`/`ΐ` name; probes/let-name-non-ascii*
  unchanged).

