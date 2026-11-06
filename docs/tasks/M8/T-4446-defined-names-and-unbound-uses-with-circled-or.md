---
id: T-4446
title: "Defined names with circled letters, marks and İ: probe which ones Excel keeps, and the decomposed use i+U+0307 of a defined İ"
milestone: M8
lane: engine
status: todo
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
- [ ] A probe case (excel-graph, definitions verified in the xlsx) covers the forms above; each cell passes, or its
      rule is recorded in references.md with an owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
