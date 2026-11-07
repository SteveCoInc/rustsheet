---
id: T-4446
title: "Defined names with circled letters, marks and İ: probe which ones Excel keeps, and the decomposed use i+U+0307 of a defined İ"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1856]
---

## Goal
Defined names whose characters T-1856 left unprobed behave as in Excel: the engine keeps, drops or refuses a defined
name starting with `Ⓐ` or a mark, or holding `İ`, as Excel does, and a use matches a defined name when Excel's does.

## Context
From T-1856 (docs/excel-semantics/references.md "Characters of a name"):
- T-1856's rule (no name starts with a mark or Ⓐ–Ⓩ) is evidenced for LET/LAMBDA names and unbound uses only. The
  engine applies it to defined names too, through `is_name_text` (expressions/utils/mod.rs, used by
  `is_valid_identifier` at load); whether Excel keeps such a definition is unprobed. xlsxwriter's `define_name` drops a
  name failing its `\w` check from workbook.xml, so write the `<definedName>` element by hand (or check `api_names`)
  and confirm in the xlsx before recalculating.
- A defined `İ` works (`=İ+1` 6) and the decomposed use `i` + U+0307 reads it (6, excel-graph 2026-11-06); check what
  the engine's name lookup (`name_key.rs`) does with that use now that the lexer reads U+0307 as a name character.
- Probe also `aⒶ`, U+1F130 and `नाम` as defined names, and a defined name starting with `ⓐ`.
- Table names and API-defined names go through `is_valid_identifier` too (tables.rs, defined_names.rs), so they now
  refuse a first `Ⓐ` or mark without evidence; probe a table name too.
- Sheet names: the engine quotes a sheet starting with such a character (`name_needs_quoting`); probe how Excel writes
  `='Ⓐ'!A1` (FORMULATEXT) and whether it loads an xlsx storing `Ⓐ!A1` unquoted.
- Other name-character checks still take only `is_alphanumeric` after the first character (expressions/utils/mod.rs
  near `is_name_text`, lexer/ranges.rs's corner and column scans): check whether a mark there should follow `is_mark`.
- Probe an enclosing mark (Me) first and later, and a negative squared letter (U+1F170) first.

## Acceptance criteria
- [x] A probe case (excel-graph, definitions verified in the xlsx) covers the forms above; each cell passes, or its
      rule is recorded in references.md with an owner.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probe probes/t4446-defined-name-characters (excel-graph, 26 cells, every definition checked in workbook.xml):
  defined `ⓐ`, U+1F130, U+1F170, `नाम`, `aⒶ`, `a`+U+20DD, `a`+U+0301, `İ` and `B1`+U+0301 work; the decomposed use
  `i`+U+0307 reads a defined `İ`; an unbound `B2`+U+0301 is `#NAME?`; sheets `Ⓐ`, `ⓑ`, `a`+U+0301 and `x`+U+20DD with
  quoted and unquoted references and their FORMULATEXT. 24/26 passed before the fix, 26/26 after.
- Refused at load (hand-built workbooks, `rustsheet-oracle recalc -b excel-graph`, not committable): a defined name
  starting with `Ⓐ` or a mark (Mn, Mc, Me), even used only through INDIRECT; a stored unquoted `Ⓐ!B1`; a structured
  reference to a table named `Ⓐ` or starting with a mark (the table itself loads while unused); the R1C1-shaped
  `C1`+U+0301. Recorded in docs/excel-semantics/references.md "Characters of a name".
- Engine: `name_needs_quoting` takes a mark after the first character as a name character, so a reference to sheet
  `a`+U+0301 is written unquoted, as Excel's FORMULATEXT shows (A20, A22). Unit test
  `sheet_names_quote_a_first_mark_or_circled_capital_but_not_a_later_mark`.
- spec2xlsx: `_define_name` writes a name that XlsxWriter's `\w` check refuses into workbook.xml instead of letting
  XlsxWriter drop it with a warning (no existing case has such a name, so no case's input changes). Cell-like names it
  still drops: follow-up T-4752. A table named `Ⓐ` on xlsx import: follow-up T-4753.
- Eval delta: +26 passing cells (new case); no regressions.
