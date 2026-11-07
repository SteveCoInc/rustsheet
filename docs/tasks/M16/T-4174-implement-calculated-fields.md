---
id: T-4174
title: "Implement calculated fields"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4505, T-4142]
---

## Goal
Calculated fields are evaluated on the aggregated sums with the engine's own parser and evaluator, in a sandbox with no references, names or I/O, bounded by the work budget.

## Context
SPEC-0032; directive 3 (reuse the parser), directive 5. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152, whose adapter loads the
  case builder's parts through the importer, which skips a cache with a calculated field; K's Top-N filter and sort are T-4165's once it loads):
  probes/t4128-pivot-spike K!A3:D11.

Owner of these visibly-off pivot cells (decision 0245; T-4130, a calculated field Dbl = Sales*2+1 the adapter skips the same way):
- probes/t4130-pivot-calc-items CF!A4:A8, CF!B3:B8, CF!C4:C8

## Acceptance criteria
- [ ] p-calc field probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items.
