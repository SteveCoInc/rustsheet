---
id: T-4611
title: "RANDARRAY lifts over an array rows or columns: RANDARRAY({2}) is one number, RANDARRAY({2;3}) a 2x1 of numbers"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
RANDARRAY lifts over an array or multi-cell `rows` or `columns`, one call per element (each element a number; that it is the call's top-left is assumed, probe it here). So
`RANDARRAY({2})` is a single number, and GROUPBY's `_xleta.RANDARRAY` cell holds one number per group record.

## Context
- probes/t4519-row-column-randarray-over-arrays (excel-graph, 2026-11-06, T-4519): `RANDARRAY({2})` is 1x1, `RANDARRAY({2;3})` 2x1 and
  `RANDARRAY({2},{3})` 1x1, each element a number (D2:E4, G2:H4, scored as `invariant`; ours D2 2x1, D3 `#N/IMPL!`, E3 16, G3 and H3 102, D4 2x3). Inside GROUPBY, `INDEX(GROUPBY(A1:A3,B1:B3,_xleta.RANDARRAY),1,2)`
  is 1x1 (D7: ours 2x1) and the Total cell `..,4,2` is a 3x1 of numbers (D8:H8: ours `#N/IMPL!`).
- probes/t4450-eta-special-forms J63:J64: `ERROR.TYPE(INDEX(GROUPBY(..,_xleta.RANDARRAY),1,2))` is one `#N/A` cell (ours spills into J64).
- Ours: `fn_randarray` (functions/math_and_trigonometry/random.rs) reads rows and columns as scalars (`randarray_extent`). Give it a
  lifting entry like SEQUENCE's (`lift/kinds/general.rs` `SEQUENCE`, `fn_sequence`'s `lift_call`, T-1834), with each lifted element charged
  the cells it builds (directive 5). Then regenerate eval/catalog/lift-kinds.json (`rustsheet-eval lift-kinds`).
- min, max and whole_number over an array are unprobed: probe them with this task (for example `ROWS(RANDARRAY(1,1,{0;5}))` and
  `RANDARRAY(1,1,{0;5},{1;6})`).
- Split from T-2644 (research: the GROUPBY/PIVOTBY eta probe, which lists the t4450 cells too) and T-4519: this task owns the engine fix for its cells.

## Acceptance criteria
- [ ] probes/t4519-row-column-randarray-over-arrays D2, D3, E3, G3, H3, D4, D7, D8, E8, G8, H8 and probes/t4450-eta-special-forms J64 pass.
- [ ] The min, max and whole_number array forms are probed and matched, and arrays.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ROW and COLUMN (T-4610).
