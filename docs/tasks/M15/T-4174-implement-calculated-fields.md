---
id: T-4174
title: "Implement calculated fields"
milestone: M15
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4173, T-4142]
---

## Goal
Calculated fields are evaluated on the aggregated sums with the engine's own parser and evaluator, in a sandbox with no references, names or I/O, bounded by the work budget.

## Context
SPEC-0032; directive 3 (reuse the parser), directive 5. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] p-calc field probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items.
