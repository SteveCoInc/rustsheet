---
id: T-4799
title: "Probe pivot Top-N percent over negative totals and value-sort ties of blanks, zeros and several errors"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4164]
---

## Goal
Evidence that settles the pivot filter and sort questions T-4164 left UNVERIFIED (docs/excel-semantics/pivots.md,
"Top-N filters at their edges" and "Value sorts: ties, blanks and errors").

## Context
- T-4164 (its Top-N probe, sheets NP50 and NBP50): a percent Top-N over a negative grand total (-8) keeps every item for
  top 50 and only D (-8) for bottom 50. Two readings fit both: the running sum compared in absolute value with the
  percentage of the absolute total, or the running sum's share of the total compared with the percentage. A shape
  that separates them: a negative total with an item whose share is negative but larger in absolute value than the
  percentage (for example A 6, B -10, total -4, top 50: the absolute reading keeps A, the share reading keeps A and B).
  Also a mixed-sign total near zero and a total of exactly 0.
- T-4164 (its value-sort probe, sheets SCd, SCa, SEd and SEa): a blank sort value sorts where a 0 would (between 5 and
  -1), but no item had a real 0 to tie with, so whether a blank equals 0 (and how the tie breaks) is open. An error
  sorts last both ways, but only one error was probed: two different errors (#N/A and #DIV/0!) and their order are
  open.
- T-4164 (its item-order probe, sheet IOD): IOD's descending order coincides with its `<items>` order, so it does
  not separate them (IOA does, for ascending).
- Shapes T-4164 left unprobed (pivots.md names each in its entry's Status): a sum or percent Top-N whose first item
  is tied (the threshold reached by the first of two equal largest items); Top-N ties and filters under several data
  fields (`iMeasureFld` 1); ties of an inner field sorted by value under an outer field; Top-N and value filters on
  an inner field when the pivot has a column field or several data fields; a hidden item of a column field and the
  totals over it; a percent or sum Top-N together with a value filter, and two value filters on one field whose
  order changes the result (T-4164 showed a value filter and a count Top-N apply in `<filters>` order); and a
  filter's `evalOrder` other than -1, or filter `id`s written against their `<filters>` position (with two filters
  on one field, and on two fields), either of which may be what that order really follows.
- T-4165 (the engine): its own readings where no probe decides, listed in pivots.md "Pivot filters: the engine's own
  readings": filters on page fields and fields off the axes, the order between two fields' value filters and an
  AutoShow beside `<filters>`, an inner-field filter beside a column field, error values under value filters and
  Top-N, a count N below 1, caption filters on number items, date conditions on non-date items, values lists and
  `dateGroupItem`s on date items, the week start of the relative week filters, DLit's drop rule on mixed fields,
  `showAll` beside a filter, and GETPIVOTDATA on a filtered pivot; the percent Top-N's absolute reading over a
  positive total with items of both signs (bottom 50 percent of 10, -20, 30: the absolute reading keeps -20 alone, the
  share reading -20, 10 and 30); an unknown filter `type`, a floored count N and AutoShow's schema defaults; the dynamic
  types other than the known ones; and the importer's new skips (a `filter@fld` past the cache, an unknown
  `customFilter@operator` or `dateTimeGrouping`).
- T-4827 (the value sorts): its own readings, listed in pivots.md "Value sorts: ties, blanks and errors":
  - a blank tied with a real 0 by the cache order;
  - several errors in cache order both ways;
  - inner-field ties by the cache order;
  - `reference@x` read as an index into the field's `<items>`, not the cache (a shape that separates them is a
    column field whose `<items>` are written out of cache order), and only its first `x` read;
  - a reference to a field on no axis or on the sorted field's own axis ignored;
  - a scope with no data reference, or one past the data fields or the items, sorting nothing, and likewise one under
    `sortType="manual"`;
  - a `showAll` item with no record sorting as 0;
  - a value sort on a column field;
  - Count and Average items with no number in the scope.
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. The engine is T-4165's and T-4827's.

## Acceptance criteria
- [ ] Probe cases with sidecars that discriminate each reading above; pivots.md entries updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4165).
