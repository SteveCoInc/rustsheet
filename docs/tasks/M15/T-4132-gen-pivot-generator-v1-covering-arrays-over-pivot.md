---
id: T-4132
title: "gen-pivot generator v1: covering arrays over pivot features and data edge pools"
milestone: M15
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4122, T-4125]
---

## Goal
`rustsheet-gen --milestone M15` produces seeded, reproducible pivot cases: pairwise coverage (3-wise for layout x subtotals x Values position x filters x show-as) over the catalog dimensions of the current stage, data from the edge pools (types, blanks, errors, case variants, dates with times, ties, many items), one pivot per sheet and many sheets per workbook.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 2. docs/architecture/synthetic-generator.md describes the existing generator. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Generator and suite `gen-pivot` committed; a first batch filled by the oracle (>= 5,000 scored cells).
- [ ] Cases tag the catalog ids they cover.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PV2+ dimensions (added as their stages open).
