---
id: T-4166
title: "Probe whether Excel loads the formula spellings our export writes (9.99e307, 1e-12, O1:O2 for O2:O1)"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
We know whether Excel reads, without repair and to the same value, the formula spellings our export writes that
differ from Excel's own: a lowercase, unpadded exponent (`9.99e307` for `9.99E+307`, `1e-12` for `0.000000000001`)
and a reversed range written in order (`O1:O2` for a stored `O2:O1`).

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
calipers roundtrip formulas_math_basic C12 and C13, formulas_datetime Sheet2!C12 and C13 and formulas_lookup C175, and
ironcalc-calc TRIGONOMETRIC A44 and PMT L7, all rewrite number literals. poi bug60858 F2 and G2 reorder reversed ranges.
Excel writes `9.99999999999999E+307` and keeps the typed digits of a literal in the file. If Excel reads our spelling
the same way, this is only a package difference. If not, it is an export bug. Use the `excel-probe` skill: a workbook
with these spellings in `<f>`, run through excel-graph, compared on value and FORMULATEXT.

## Acceptance criteria
- [ ] An excel-graph probe records, for each spelling, whether Excel loads it and what its value and FORMULATEXT are.
- [ ] The finding is in docs/excel-semantics/ (xlsx-storage.md).
- [ ] An engine task is filed if any spelling changes meaning or is refused.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the export.
