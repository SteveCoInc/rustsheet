---
id: T-4714
title: "LARGE/SMALL/MEDIAN over a mixed-sheet reference grid: the first element's sheet decides #VALUE! vs #N/A"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
LARGE, SMALL and MEDIAN over an array of references on both the formula's sheet and another give `#VALUE!` when the
first reference element is on the formula's own sheet and `#N/A` when it is on another sheet.

## Context
Evidence: T-4697, probes/t4697-large-small-median-mixed-sheet-refs; SPEC-0014 rule 5; lookup.md. Today
`RefGrid::order_statistic_error` (T-4570) answers `#N/A` when any reference is on another sheet.
Owner of the visibly-off cells Local!H1, H3, H5, H9 of that case.

## Acceptance criteria
- [ ] The four cells match Excel (`#VALUE!`), H2, H4, H6, H7, H8, H13:H15 keep passing, baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
