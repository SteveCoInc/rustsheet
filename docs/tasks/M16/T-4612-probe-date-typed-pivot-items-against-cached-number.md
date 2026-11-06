---
id: T-4612
title: "Probe date-typed pivot items against cached number items, and which casing wins when the first record is upper case"
milestone: M16
lane: research
status: todo
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
- [ ] Filled probe separates the casing rules; pivots.md states the rule.
- [ ] The appended date items' type is recorded from the saved cache.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine refresh (T-4141).
