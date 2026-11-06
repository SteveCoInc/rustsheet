---
id: T-4359
title: "GROUPBY's value sort on an intransitive run: a parent's value counts the rows that reached it but no leaf"
milestone: M7
lane: engine
status: done
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
- [x] D56 and E56 pass, or stay known failures with the reason recorded; the rule is confirmed out of sample.
- [x] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's column leaves (T-4358).

## Outcome
- Rule: a GROUPBY parent holds the rows that reached its class (a row that reached no leaf under it included, the
  extra rows of a leaf no row reaches not), and every row when no row reached it. A value sort orders the parents by
  the function of those rows, and a subtotal line shows it, under a value sort and a key sort alike
  (docs/excel-semantics/coercion.md "GROUPBY on an intransitive run").
- Evidence: probes/t4359-groupby-value-sort-parents (28 seeded runs, two and three fields; SUM ascending and
  descending, AVERAGE, SUM with subtotals), written with three hypotheses' predictions before the fill
  (docs/research/t4359/hypotheses.tsv: R0 the groups' rows 24/84, R1 the reached rows 52/84, R2 both 65/84). Its
  subtotal cells gave R3 (R1, every row for a class none reached; 84/84), chosen after the fill. Hold-out
  probes/t4359-groupby-value-sort-holdout (30 fresh runs, 12 two-field, 18 three-field), the engine's R3 predictions
  committed in 063704e15 before its fill: 120/120 value-sort cells (B:E). Its column F (key sort with subtotals,
  predicted with the old union) was 0/30; R3, frozen before the fill, gives 30/30, and the engine now does. A
  structural choice confirmed out of sample (decision 0188 R4), not a fit; no FITTED tag.
- Code: `Grouping::class_rows` (grouping.rs) over per-field reached-row maps built in `Grouping::new` from T-4378's
  descent paths; `sort_by_aggregate`'s parents and `line_parts`' subtotals (`subtotal_rows`, replacing `rows_of`)
  both read it. Unit test `a_parent_holds_the_rows_that_reached_its_class`. Builds on T-4378 (#3403), merged in.
- Eval delta: probes fixed 2 (t4344 Calc!D56, E56), new coverage 2 cases (1002 cells, all pass), regressions 0.
- Follow-up: T-4388 (PIVOTBY's row-axis parents and a column subtotal of a class no record reached, unprobed).
