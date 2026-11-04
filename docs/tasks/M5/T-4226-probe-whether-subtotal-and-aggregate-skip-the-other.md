---
id: T-4226
title: "Probe whether SUBTOTAL and AGGREGATE skip the other cells of a CSE array formula that calls SUBTOTAL"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for whether SUBTOTAL, and AGGREGATE options 0-3, skip the non-anchor cells of a CSE (legacy array) formula
that calls SUBTOTAL, as they skip the spilled cells of a dynamic-array one, and the engine matching it.

## Context
- Follow-up of T-4194 (excel-fidelity review, MINOR). probes/t4194-subtotal-nested-skip shows a spilled cell of
  `=MAP(..,LAMBDA(r,SUBTOTAL(9,r)))` carries its anchor's nested-SUBTOTAL flag (numbers.md "SUBTOTAL and AGGREGATE over
  hidden and filtered rows"). The engine stores the other cells of a CSE block as `Cell::SpillCell` too, so
  `cell_formula_is_one_of` (crates/rustsheet-engine/src/functions/subtotal.rs) now skips them as well; Excel's
  behavior there is unprobed.
- Probe (excel-probe skill, excel-graph, tag M5): a 3-cell CSE block `{=SUBTOTAL(9,OFFSET(A1,ROW(A1:A3)-1,0))}` (case
  `array_formulas`, see docs/architecture/eval-harness.md) and a CSE block of a plain formula as a control, each read
  cell by cell and as a range by SUBTOTAL 9 and 109 and AGGREGATE(9,0) and AGGREGATE(9,4).

## Acceptance criteria
- [ ] The probe case is filled on excel-graph and passes; numbers.md states the rule and drops "Unprobed: a CSE array
      formula's other cells".
- [ ] If Excel reads those cells, `cell_formula_is_one_of` follows only a dynamic spill to its anchor (one helper).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other nested-call forms (T-4194).
