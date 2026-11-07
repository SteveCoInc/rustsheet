---
id: T-4150
title: "A carriage return in a shared string survives load and save instead of becoming literal _x000d_"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text containing a carriage return (`\r`, as in `"Rich Text\r\nTest"`) has the same value after load and save.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
poi/60289.xlsx TABLE!B2 is `"Rich Text\r\nTest"`. After load and save it reads `"Rich Text_x000d_\nTest"`. Excel writes a
CR in a shared string as `_x000D_` and the importer decodes it (`decode_xlsx_escapes`), so the export side, or a
decode on one path only, is wrong. Find which one. Fix the escape with the existing helper (directive 3), and cover
the other `_xHHHH_` code points Excel escapes, and a literal `_x` in text (Excel writes `_x005F_x`).

## Acceptance criteria
- [x] poi/60289.xlsx TABLE!B2 keeps its value through load and save, by a test.
- [x] A test saves and reloads text containing `\r`, a tab, U+0001 and a literal `_x000D_`, and gets the same text
      back.
- [x] `make check` green; reviewers approve.

## Out of scope
- `xml:space="preserve"` (T-0497).

## Outcome
Cause: the importer's `inlineStr` branch (and the inline formula-value branch) took the text without `decode_xlsx_escapes`, so poi/60289.xlsx TABLE!B2 (`Rich Text_x000d_` inline) loaded with a literal `_x000d_`; export then escaped the underscore. Export was right. Both branches now decode with the existing helper. Test `tests/text_escape_round_trip.rs`: the CR, tab, U+0001 and literal `_x000D_` texts round-trip, and inline escapes decode and round-trip. The real file is not fetched in this checkout; the test uses its cell text. The decoding is UNVERIFIED against Excel (semantics entry in xlsx-storage.md). Eval delta: none expected (roundtrip-poi not fetched here).
