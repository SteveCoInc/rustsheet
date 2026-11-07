---
id: T-4753
title: "xlsx import of a table named Ⓐ or starting with a mark, which Excel loads while no formula uses it"
milestone: M8
lane: engine
status: todo
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
- [ ] A test loads a workbook with a table named `Ⓐ` (unused) and shows the engine keeps the sheet's cells and the table,
      or the deviation is recorded with evidence in references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
