---
id: T-4226
title: "Probe whether SUBTOTAL and AGGREGATE skip the other cells of a CSE array formula that calls SUBTOTAL"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4227]
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

## Handoff (resumed)
- The task names a case key `array_formulas`; it does not exist. `rg array_formulas` finds nothing in
  `tools/src/rustsheet_tools/spec2xlsx.py`, `crates/rustsheet-eval/src/case.rs`, the oracle backends or
  `docs/architecture/eval-harness.md`. The case format can only write dynamic-array single-anchor formulas
  (`write_dynamic_array_formula(row, col, row, col, ..)`) and `legacy_formulas` (single-cell, non-array).
- A multi-cell CSE block (`{=...}` over A1:A3) therefore needs harness work first: a per-sheet key in the
  case schema, `spec2xlsx` writing `write_array_formula(first, last, ..)`, the engine adapter entering it
  (it must create `Cell::SpillCell` members), and the excel-graph fill writing it. That is its own
  harness-lane task (size M) plus an eval-harness.md entry; split it out, then do the probe.
- Nothing else was tried; no code changed. excel-graph is available (`rustsheet-oracle status` OK).
- Resumed by an opus run, 2026-11-04: the notes hold (no case key writes a multi-cell `<f t="array">`). The harness
  work is split out as T-4227; this probe waits on it (`depends_on`). `array_formulas` in the Context above is the
  key T-4227 adds.

## Handoff
Sonnet run found the probe cannot be written yet: the `rustsheet-case/v1` sheet spec (`SheetSpec` in
crates/rustsheet-eval/src/case.rs, docs/architecture/eval-harness.md) has no way to declare a CSE
(legacy array) block. The "case `array_formulas`" the Context names does not exist as a sheet field
(searched case.rs, spec2xlsx.py, eval/suites: only `legacy_formulas`, which are non-array `<f>`).
Next steps: (1) add an `array_formulas: [{ref, formula}]` sheet field written by spec2xlsx as
`<f t="array" ref=...>` (with a note in eval-harness.md), have the engine adapter enter it with
`Model::set_user_array_formula`, and check that the excel-graph backend (Graph workbook API) can load such a
file; (2) write the probe per the Context; (3) fill with `rustsheet-oracle fill -b excel-graph --suite probes`
(excel-graph is OK); (4) numbers.md rule + engine change. Consider splitting (1) into its own harness-lane task.
