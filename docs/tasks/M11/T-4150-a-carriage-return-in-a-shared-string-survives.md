---
id: T-4150
title: "A carriage return in a shared string survives load and save instead of becoming literal _x000d_"
milestone: M11
lane: engine
status: in-progress
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
- [ ] poi/60289.xlsx TABLE!B2 keeps its value through load and save, by a test.
- [ ] A test saves and reloads text containing `\r`, a tab, U+0001 and a literal `_x000D_`, and gets the same text
      back.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `xml:space="preserve"` (T-0497).
