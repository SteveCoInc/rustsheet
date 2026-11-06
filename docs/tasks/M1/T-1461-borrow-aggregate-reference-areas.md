---
id: T-1461
title: "Borrow AGGREGATE's reference areas instead of copying a LET-bound union"
milestone: M1
lane: perf
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AGGREGATE's reference form (functions 1-13) reads a LET-bound union's areas without copying them, as SUM and SUBTOTAL do.

## Context
T-1162 performance review: `aggregate_reference_form` (functions/aggregate.rs) calls `reference_areas`, which turns the `AreaList::Shared` of a LET-bound union into a `Vec` (`AreaList::into_vec`), so `AGGREGATE(9,4,u,u,u,...)` copies up to `MAX_AREAS` ranges per argument before `check_area_cells` runs. Bounded, short-lived, but it is the copy T-1162 removed elsewhere. `reference_ranges` returns `Ranges`, which derefs to `&[Range]` without a copy; if `reference_areas` then has no caller, remove it (one definition per concept).

## Acceptance criteria
- [ ] `aggregate_reference_form` walks borrowed areas (`reference_ranges` or equivalent); the existing AGGREGATE tests, `aggregate_of_a_union_obeys_the_area_cell_bound` included, pass unchanged.
- [ ] Errors stay as they are: AGGREGATE's reference-form error for a non-reference argument (probes/aggregate-args) is unchanged; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The seen-range records of long walks (T-1463).

## Outcome
`Model::reference_areas` now returns the `AreaList` (made `pub(crate)`; it derefs to `&[Range]`) instead of an
`into_vec` copy, and `aggregate_reference_form` iterates it by reference, so a LET-bound union's `Arc<[Range]>` is read
without a copy. `reference_ranges` was not used: its `Ranges::from_list` adds a cross-sheet `#VALUE!` and an
empty-intersection error that AGGREGATE's reference form does not give today, which would change behavior; borrowing
`reference_areas` keeps every error as it was. Existing AGGREGATE tests (46) pass unchanged; no eval delta.
