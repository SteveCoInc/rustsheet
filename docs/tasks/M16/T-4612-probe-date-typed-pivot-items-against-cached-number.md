---
id: T-4612
title: "Probe date-typed pivot items against cached number items, and which casing wins when the first record is upper case"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md settles two parts of T-4582's item findings:
- which property picks the label of a merged text item (the first record's casing, the listed casing excluded, or the lowercase one);
- whether the items Excel appends for date-formatted source values are date-typed (`<d>`) in the saved cache.

## Context
T-4582 (probes/t4582-pivot-item-edges):
- J1: `east` won. It was the first record's casing, the lowercase one and the ordinal maximum all at once.
- J3/J4: date-formatted values did not reuse the cached `<n>` items, and Excel appended x = 3–5 instead (Excel-fidelity review of T-4582).

Cases: "lowercase" and "ordinal maximum" are one candidate for ASCII variants, so separate first-record, listed and lowercase:
- first record `EAST` (formula), later `east` literal;
- first record `East` as a formula (cached `<m/>`), later `EAST` and `east` literals, so the listed casing is `EAST`, the first record's `East` and the lowercase `east`.

For dates, the sidecar's `cache_fields` need the shared-items type of the appended items. Extend the T-4123 capture with it if it lacks it. T-4141 depends on the answer.

## Acceptance criteria
- [x] Filled probe separates the casing rules; pivots.md states the rule.
- [x] The appended date items' type is recorded from the saved cache.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine refresh (T-4141).

## Outcome
probes/t4612-pivot-date-items-and-casing (excel-graph, 2026-11-06) holds eight builder-v2 pivots. The
oracle capture now records each cache field's saved shared items with their type (`shared_items`,
`<d>` apart from `<n>`) and the `<sharedItems>` attributes (`shared_flags`):
crates/rustsheet-eval/src/xlsx_snapshot/pivots.rs, budget-charged, with tests. Recorded in
docs/excel-semantics/pivots.md:
- **Casing (C1-C3):** the first source record's casing wins when it is a formula the old cache holds as
  `<m/>` (every probed case; a literal first record is T-4654's): C1 shows `EAST` over a listed and
  lowercase `east`; C2 and C3 show `East` over a listed `EAST` or `east`. `FieldReader::push` already did
  this; its comment now cites the probe, with a unit test.
- **Date items (T1-T5):** the items Excel appends for date-formatted values are `<d>`. `h:mm` and
  `[h]:mm` count as dates (T2, T3), and a date and an equal plain number are two items (T5). The engine
  already matched; unit tests added.
- **Flags:** a date beside numbers is a mixed type (`containsMixedTypes="1"`, T5), and a field with a
  date carries no number flags (T5, T4). `SharedItemsFlags::of` now does both. A date beside a string
  and no number is T-4654's (T4 also held stale numbers).
- Open, filed as T-4654: a literal first record's casing, date+string without numbers, how stale items count in the flags (`containsNonDate="0"` beside stale numbers,
  `containsBlank="1"` from a stale `<m/>`), a `maxDate` below `minDate` in every date field, and the
  saved shared-item order (source order first, stale items after it), which the export (T-4176) needs.

