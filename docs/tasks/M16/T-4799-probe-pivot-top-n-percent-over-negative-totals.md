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
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. The engine is T-4165's.

## Acceptance criteria
- [ ] Probe cases with sidecars that discriminate each reading above; pivots.md entries updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4165).
