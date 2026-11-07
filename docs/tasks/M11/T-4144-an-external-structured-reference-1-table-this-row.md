---
id: T-4144
title: "An external structured reference [1]!Table[[#This Row],[Col]] loads as a reference, not #REF!"
milestone: M11
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4140]
---

## Goal
A structured reference into a table of another workbook, `[1]!Table3[[#This Row],[Resource Name]]`, loads as an
external reference and is saved as stored, instead of turning into `#REF!`.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
corpus-bench 33094 Sheet1!B5 `VLOOKUP([1]!Table3[[#This Row],[Resource Name]],[1]Data!E:J,6,0)` comes back as
`VLOOKUP(#REF!,'[1]Data'!E:J,6,0)` (62 cells). Find whether the parse on import or the print on export loses it. Look for
an existing rule in docs/excel-semantics/ on external structured references. If there is none, use the `excel-probe`
skill for how Excel evaluates it against the cached external values, and record it.

## Acceptance criteria
- [ ] corpus-bench 33094's external structured references survive load and save with their stored text, by a test.
- [ ] Their value is Excel's (cached value or probe), or a filed follow-up names the evaluation gap.
- [ ] Any quirk found goes into docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Plain external references (T-4140).
