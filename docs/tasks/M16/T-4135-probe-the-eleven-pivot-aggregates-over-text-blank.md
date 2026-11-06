---
id: T-4135
title: "Probe the eleven pivot aggregates over text, blank and error data"
milestone: M16
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
Evidence for each of Sum, Count, Average, Max, Min, Product, Count Numbers, StdDev, StdDevp, Var and Varp when the value field holds text, blanks, booleans or errors: which records count, which error shows (and whether `showError`/`errorCaption` change it), empty intersections, single-record StdDev/Var, and summation order on non-representable decimals (cache order vs display order).

## Context
SPEC-0015 unknowns 2, 6, 15 and 16 overlap (T-2085). libreoffice.md: errors poison all but Count. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Probe cases with sidecars; pivots.md entries.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4135-pivot-aggregates (excel-graph, 2026-11-06) holds twelve builder-v2 pivots: one per aggregate
over six value fields, and an errorCaption pivot. Recorded in docs/excel-semantics/pivots.md:
- **Skipped values:** text, blanks and booleans are skipped by every aggregate except Count. Booleans are
  not numbers.
- **Count and CountNums:** Count counts every non-blank value, errors included. CountNums counts numbers
  and is not poisoned by errors.
- **Errors:** an error poisons every other aggregate, in its cell and in every total over it.
- **One value:** StdDev and Var are `#DIV/0!`, StdDevp and Varp 0.
- **Empty intersections:** blank for every aggregate.
- **Totals:** a Sum total is not the record-order or display-order sum. Corrected and settled by
  T-4589 and T-4614: a total over one axis field accumulates item by item in display order. A Sum
  row total across a column field, as here, equals the sum of the row's cells (T-4616).

  See pivots.md "Summation order of Sum, Average and Product totals".
- **errorCaption:** replaces every error value, `#DIV/0!` included.

**Left open:** T-4589 and T-4614 showed that EC's Count grand totals carry a real error when the data
fields mix aggregates; the caption only displays it. The orders that remain are T-4614's,
and so are StdDev and Var totals.

