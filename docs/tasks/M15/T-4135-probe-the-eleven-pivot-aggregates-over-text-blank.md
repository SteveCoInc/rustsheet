---
id: T-4135
title: "Probe the eleven pivot aggregates over text, blank and error data"
milestone: M15
lane: research
status: todo
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
- [ ] Probe cases with sidecars; pivots.md entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
