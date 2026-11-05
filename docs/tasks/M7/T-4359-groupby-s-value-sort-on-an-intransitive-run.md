---
id: T-4359
title: "GROUPBY's value sort on an intransitive run: a parent's value counts the rows that reached it but no leaf"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4344]
---

## Goal
GROUPBY sorted by a value column orders its parents as Excel does on an intransitive run, where some rows reach a
first-field class but no leaf under it.

## Context
- probes/t4344-groupby-descending-search Calc!D55:E66 (T-4344): two-field runs, SUM of 2^(position-1), sort_order
  3 (D) and -3 (E). A = ab, B = ab&U+00AD, C = U+00AD&ab. Engine 22 of 24.
- D56 and E56: rows Bx, Ax, Ax, By, Cy (values 1, 2, 4, 8, 16). Groups (B,x) 1, (B,y) 8, (A,x) 6; row 5 (C,y)
  reaches class A of the first field (C = A) and then no y class, so it is in no group. Excel orders ascending
  (B,x), (B,y), (A,x) and descending (A,x), (B,y), (B,x): the parents by B 9 < A 22, where A's 22 is 2 + 4 + 16,
  the rows that reached class A, the dropped row 5 included. `Grouping::sort_by_aggregate` gives a parent the rows
  of its groups (A 6), so it puts A first ascending.
- Check the rule on more runs (a row dropped at the second field of three; AVERAGE, whose value the dropped row
  moves) before implementing; a table relationship (B and C of the same rows) passes all 24.
- Owned cells: probes/t4344-groupby-descending-search (2): Calc!D56, Calc!E56.

## Acceptance criteria
- [ ] D56 and E56 pass, or stay known failures with the reason recorded; the rule is confirmed out of sample.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's column leaves (T-4358).
