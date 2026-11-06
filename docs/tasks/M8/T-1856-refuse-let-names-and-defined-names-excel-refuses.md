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
- Rule (references.md "Characters of a name"): a name in a formula may not start with a mark or an uppercase circled
  letter Ⓐ–Ⓩ (U+24B6–U+24CF). Excel refuses the workbook at load, as a LET or LAMBDA name and as an unbound use
  (`=Ⓑ+1`). After the first character every mark loads. The task's premise was half an artifact: T-1431's `İ` refusal
  was spec2xlsx writing the bound name without `_xlpm.` (its Python `\w` misses `İ`, `Ⓐ` and marks); with the prefix
  written by hand, `LET(İ,1,İ+1)` is 2. `Ⓐ` is refused however it is stored. Round 1's fidelity review caught a first
  version built on unprefixed probes; the rule above comes from prefixed re-probes only.
- Probes: about 80 single-cell workbooks on excel-graph (2026-11-06). The 37 that load are committed as
  probes/t1856-name-characters (the forms the engine must accept). The refused ones (`Ⓐ`, `Ⓑ`, `Ⓜ`, `Ⓩ`, `Ⓐx`,
  `LAMBDA(Ⓐ,..)`, `=Ⓐ+1`, `=Ⓑ+1`, a first U+0345, U+093E, U+0301 or U+0307) have no oracle cells and cannot be
  committed (T-1387's precedent).
- Engine: `is_letter_name_start` (expressions/utils/mod.rs) refuses those first characters in `is_name_text` and the
  lexer's identifier start, so a refused name is a parse error (`#ERROR!`). `is_mark` lets a later character be any
  mark (`a` + U+0301 and `i` + U+0307 were `#ERROR!`). `is_name_text` checks the characters as typed, not their
  uppercase, so `ǰ` and `ΐ` (`#ERROR!` before) are names. Unit test
  `names_starting_with_a_mark_or_an_uppercase_circled_letter_are_refused` (11 refused, 33 accepted).
- A sheet name starting with such a character is now quoted in written formulas (`name_needs_quoting`), so `'Ⓐ'!A1`
  reads back (fidelity round 2: `FORMULATEXT` gave `=Ⓐ!A1+1`, which the lexer now refuses); unit test
  `a_sheet_name_starting_with_a_refused_name_character_is_quoted`. Excel's own form is unprobed (T-4446).
- Defined names: Excel's handling of a defined name starting with `Ⓐ` or a mark is unprobed (xlsxwriter drops them),
  and the engine now refuses them when defined through the API (`is_valid_identifier`), as it does table names. Filed as T-4446, with the decomposed use of a defined `İ`.
- Formula entry vs xlsx load: the oracle only loads xlsx; the engine applies the rule to both, as for T-1387's
  duplicate names.
- Eval delta: probes +1 case (t1856-name-characters, 37/37 pass; without
  this change its `ǰ`, `ΐ`, `a` + U+0301 and `i` + U+0307 cells are `#ERROR!`). 0 regressions.
