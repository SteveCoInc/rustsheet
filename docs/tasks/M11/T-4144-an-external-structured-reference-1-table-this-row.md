---
id: T-4144
title: "An external structured reference [1]!Table[[#This Row],[Col]] loads as a reference, not #REF!"
milestone: M11
lane: engine
status: done
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
- [x] corpus-bench 33094's external structured references survive load and save with their stored text, by a test.
- [x] Their value is Excel's (cached value or probe), or a filed follow-up names the evaluation gap.
- [x] Any quirk found goes into docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Plain external references (T-4140).

## Outcome
- Cause: the parse, not the print. A loaded structured reference whose table is not in the workbook became a bare
  `#REF!` node (T-2197's rule for this workbook's tables), so `[1]!Table3[...]`, a table of the linked workbook,
  lost its text in the model: on main, 33094 Sheet1!B5 reads `=VLOOKUP(#REF!,[1]Data!E:J,6,0)` after import.
  (The 2026-11-04 census loss no longer shows in the round-trip saves of 33094 because the export now patches cells
  it does not rewrite from their source `<f>`, but the model, FORMULATEXT and every rewritten cell still lost it.)
- Fix: `is_external_table_name` (structured_reference.rs). A stored or engine-re-entered structured reference to a
  linked workbook's table is kept as a `StructuredReferenceKind` lowered to `#REF!`: it evaluates `#REF!` as before
  (references.md, VERIFIED on corpus-bench 58994) and prints and saves as written. Typed, it is still refused.
  33094 B5 now reads `=VLOOKUP([1]!Table3[@[Resource Name]],[1]Data!E:J,6,0)` and saves
  `[1]!Table3[[#This Row],[Resource Name]]`.
- Value: `#REF!` as before, by the rule VERIFIED for a link part caching no cells (58994). 33094's link part caches
  cells, its cached value is 20 and it has no oracle value, so that shape is unprobed: filed T-5036 (research, sonnet)
  to probe it. FORMULATEXT of such a cell stays unprobed (noted in references.md).
- Tests: test_external_links `structured_reference_into_a_linked_workbook_is_ref` checks the kept text;
  rustsheet-xlsx tests/external_reference_export.rs `external_structured_references_are_saved_as_written` (33094's
  formula over the committed external-link-cached-lookups fixture) checks value and saved text. Both fail on main.
- Eval delta: cell suites unchanged; roundtrip-corpus-bench fewer failing items in 3 cases (132-4 217 → 216,
  248-48 359 → 349, 575-15 963 → 962), re-recorded by `make eval-baseline`; after #4007 these are the `#R1/all` keys
  (216 → 215, 358 → 348, 963 → 962), re-recorded by `rustsheet-eval roundtrip --update-baseline`.
- Follow-up: T-5036.
