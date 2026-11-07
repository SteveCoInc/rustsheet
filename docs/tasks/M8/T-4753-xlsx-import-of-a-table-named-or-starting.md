---
id: T-4753
title: "xlsx import of a table named Ⓐ or starting with a mark, which Excel loads while no formula uses it"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
An xlsx holding a table whose name starts with `Ⓐ` or a mark loads into the engine as Excel loads it (the table kept,
its cells readable), and a structured reference to it behaves as Excel's does.

## Context
Found by T-4446 (docs/excel-semantics/references.md "Characters of a name", "Table names"): Excel for the web loads a
workbook with a table named `Ⓐ`, `ⓐ` or U+0301 + `a` while no formula uses it, and refuses `SUM(Ⓐ[h])` at load. The
engine's table-name check goes through `is_valid_identifier` (tables.rs), which refuses `Ⓐ` and a first mark; what
rustsheet-xlsx's import then does with such a table (drop it, drop the workbook, keep it) is untested. spec2xlsx cannot
write the table (XlsxWriter's `add_table` refuses the name), so a case needs the table part patched in by the builder,
or a hand-built workbook under crates/rustsheet-xlsx/tests.

## Acceptance criteria
- [x] A test loads a workbook with a table named `Ⓐ` (unused) and shows the engine keeps the sheet's cells and the table,
      or the deviation is recorded with evidence in references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- No engine change was needed: rustsheet-xlsx's table import does not run `add_table`'s name check, so a table part
  named `Ⓐ`, `ⓐ` or U+0301 + `a` loads with the table (sheet, ref) and the sheet's cells, as Excel loads it.
- tests/table_part_names.rs (patching tests/example.xlsx's table part): `a_table_named_with_a_circled_letter_or_a_mark_loads`
  and `a_structured_reference_to_a_circled_or_marked_table_name` (a loaded `SUM(ⓐ[Year])` reads the table, 5935, as
  Excel's `SUM(ⓐ[h])` does; `SUM(Ⓐ[Year])` and the mark twin, which Excel refuses at load, are `#ERROR!` with their
  text kept, the engine's stand-in for every refused name).
- references.md "Characters of a name", Table names: the import behavior recorded (no longer "untested").
- Eval delta: none (spec2xlsx cannot write these tables; Rust tests only).
