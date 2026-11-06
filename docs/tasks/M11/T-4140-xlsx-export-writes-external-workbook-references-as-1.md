---
id: T-4140
title: "xlsx export writes external-workbook references as [1]Sheet!A1, not '[1]Sheet'!A1"
milestone: M11
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A formula that refers to another workbook, `[1]Sheet!A1`, is saved in the form Excel writes, so the
saved file's formula still points at external link 1.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
The export prints the book index inside the sheet quotes: `VLOOKUP(C2,[1]Sort!$B:$C,2,FALSE)` comes back as
`VLOOKUP(C2,'[1]Sort'!$B:$C,2,FALSE)`, so Excel would read a sheet literally named `[1]Sort`. This hits 60+ corpus
files, for example corpus-bench 59932 (16,998 cells), 566-40, 14240, 13284 and 47699, poi ref-56737, ref2-56737,
bug67784 and 52575_main, and the fixtures `external-link-*` cases. Excel's stored form, from those Excel-saved inputs, is
`[1]Sheet!A1`, and `'[1]My Sheet'!A1` when the sheet name needs quotes (quotes go around the whole `[n]Sheet` token).
The quoting is in the formula stringifier's sheet-name path
(`crates/rustsheet-engine/src/expressions/utils/mod.rs` `quote_name` and its callers in `parser/stringify.rs`).
Without T-4143 the external link parts are still missing from the output. This task fixes only the formula
text, which is also what FORMULATEXT-style printing of external refs needs.

## Acceptance criteria
- [x] Load and save of the fixtures `external-link-*` workbooks and poi ref-56737 writes every external reference
      exactly as stored in the source (a test that compares the `<f>` text).
- [x] A sheet name that needs quoting is written as `'[1]My Sheet'!A1`, with a test, and the source evidence is cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing the externalLink parts (T-4143).

## Outcome
- `quote_name` (expressions/utils/mod.rs) leaves another workbook's `[book]Sheet` bare when the sheet part
  needs no quotes and quotes the whole token otherwise (`[1]Sort`, `'[1]XA Data Pull'`, `'[1]56737'`).
  That is ADDRESS's evidenced rule (T-0984), which moved out of `address_sheet_text` into
  `quote_name`, so there is one definition. The display, FORMULATEXT, the stored R1C1 text and the xlsx
  export all print through it.
- A 3-D span of the other workbook's sheets, `[1]Uses:Defines`, is bare unless either end needs quotes,
  as a local span is (`sheet_span_text`). That is the form fixtures/external-link-cached-3d-span holds;
  Excel's own saved form for it is UNVERIFIED (references.md).
- Tests:
  - crates/rustsheet-xlsx/tests/external_reference_export.rs loads and saves the seven
    `external-link-*` fixtures; every `<f>` with `[1]` comes back as the source holds it.
    External-link-cached-ranges is poi ref-56737's Excel-written workbook, but its `Refs` sheet was
    rewritten, so it is not Excel-written.
  - `test_quote_name` covers the bare, quoted, escaped and span cases.
  - test_implicit_intersection_export's external case now expects `[1]Uses!`, and T-1654's
    `span_of_cached_sheets_round_trips` the bare span (it pinned the quoted form, which no evidence backed).
- Excel evidence: references.md, new entry "A formula's reference to another workbook is written
  `[1]Sheet!A1`" (corpus-bench 59932 and 52450, Excel-saved). The T-1654 note there about the quoted
  form is updated.
- Eval delta: none (every suite unchanged, 0 regressions; roundtrip-fixtures still 12 failing items, which the
  external-link parts dropped by T-4143 account for).
