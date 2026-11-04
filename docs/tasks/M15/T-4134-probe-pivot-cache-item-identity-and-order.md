---
id: T-4134
title: "Probe pivot cache item identity and order"
milestone: M15
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
docs/excel-semantics/pivots.md records how Excel builds a field's items on refresh: case-variant text (one item? which casing shows: first seen?), type order (numbers, text, booleans, errors, blank), dates with and without times, numbers that differ in the last bits, `(blank)` placement, and text that looks like a number.

## Context
libreoffice.md §2 (case-insensitive dedup, numbers < text < errors < blank) and oss-engines.md (gridcraft's class order) are hypotheses to confirm or refute. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars covering each question; semantics entry with evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
