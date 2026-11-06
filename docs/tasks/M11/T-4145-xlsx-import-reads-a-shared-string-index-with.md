---
id: T-4145
title: "xlsx import reads a shared-string index with surrounding whitespace (<v>2 </v>) as that index, not 0"
milestone: M11
lane: engine
status: todo
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
- [ ] poi/bug69769.xlsx Tabelle1!B2 loads as "Mustermann", by a test.
- [ ] The importer and the snapshot reader share the index parse.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An out-of-range index (already handled).
