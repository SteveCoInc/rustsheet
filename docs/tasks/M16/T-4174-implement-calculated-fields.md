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

Owner of these visibly-off pivot cells (decision 0245; T-4505, the same cause: the adapter skips a cache with a
calculated field, so each pivot area is empty; the rules they show are pivots.md "Calculated fields as Excel for
the web renders them"):
- probes/t4505-pivot-calc-fields LV!A3:D12, LC!A3:D8, CNT!A3:F8, DZ!A3:D12, NV!A3:D8, K!A3:D12, FA!A3:F8,
  NM!A3:D8, CC!A3:D12
- probes/t4505-pivot-calc-field-functions FX!A3:C8, FR!A3:C8, FL!A3:C8, FK!A3:C8, FM!A3:C8, FN!A3:C8,
  FV!A3:C8, ET!A3:C8, EC!A3:C8, ES!A3:C8, EF!A3:C8

Owner of these visibly-off cells (decision 0245; T-2085, a calculated field Double = Sales*2 the adapter skips the
same way, so the pivot is empty and GETPIVOTDATA on it is `#REF!`; Excel answers by the field's name or caption,
web.md "GETPIVOTDATA on other pivot shapes (T-2085)"):
- probes/t2085-getpivotdata-layouts CF!A3:B6, Q!G1:G3

Excel for the web refuses a file whose calculated field calls NOW, TODAY, RAND or INDIRECT, holds an array
constant or a cell reference, or names an unknown field (pivots.md, T-4505): what the importer does with such
a file is this task's choice, recorded as a DEVIATION if it opens it.

## Acceptance criteria
- [ ] p-calc field probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items.
