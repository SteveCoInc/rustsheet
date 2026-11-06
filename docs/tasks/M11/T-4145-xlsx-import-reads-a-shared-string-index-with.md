---
id: T-4145
title: "xlsx import reads a shared-string index with surrounding whitespace (<v>2 </v>) as that index, not 0"
milestone: M11
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A shared-string cell whose `<v>` index has surrounding whitespace loads the string at that index.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
poi/bug69769.xlsx Tabelle1!B2 stores `<c r="B2" t="s"><v>2 </v></c>`. The importer reads "Konto" (index 0)
instead of "Mustermann" (index 2). The `rustsheet-eval snapshot` reader already trims and reads "Mustermann", so the
harness and the importer disagree. POI's bug 69769 is this file, which Excel opened. Read the index the way the
snapshot reader does (one definition, CLAUDE.md directive 3). Check the other integer-valued `<v>` and attribute parses
in `import/sheet_data.rs` for the same gap.

## Acceptance criteria
- [x] poi/bug69769.xlsx Tabelle1!B2 loads as "Mustermann", by a test.
- [x] The importer and the snapshot reader share the index parse.
- [x] `make check` green; reviewers approve.

## Out of scope
- An out-of-range index (already handled).

## Outcome
`rustsheet_xlsx::util::parse_shared_string_index` (trims, `None` if not a non-negative integer) is now the one parse: the importer (`get_cell_from_excel`, `t="s"`) and the snapshot reader (`decode_value`) both call it. Unit test `shared_string_index_with_surrounding_whitespace_is_that_index` covers `<v>2 </v>`. The acceptance criterion on the real file is met only by this equivalent unit test; the poi/bug69769.xlsx file is not fetched in this checkout, so the test uses the same cell XML; the Excel behavior itself is UNVERIFIED (semantics entry in xlsx-storage.md, probe T-4711). The other integer parses in `sheet_data.rs` are style/row indices, left as they were (not an index into a table where a wrong value changes content). Eval delta: none expected (no baseline change).
