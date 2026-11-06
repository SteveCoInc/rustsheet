---
id: T-4528
title: "ISEVEN and ISODD of an array expression over a range (C1:C5+0, -C1:C5, IF over a range) and their legacy-formula @ forms: probe and match"
milestone: M5
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ISEVEN and ISODD of an array computed from a range, and their legacy (pre-dynamic-array) formula
forms, have Excel evidence, and the engine matches it, so the UNVERIFIED sub-rules of
docs/excel-semantics/information.md "ISEVEN/ISODD over a range" become VERIFIED or are rewritten from
the evidence.

## Context
- The entry is VERIFIED for a bare range (one `#VALUE!`, no spill, no implicit intersection) and an
  array constant (`ISEVEN({2,3})` spills), from probes/is-functions-array-lift (Calc!A15, B15, D15,
  E15, J2, K2, L3, M30:N30, M32:N32, M34:P34). Two sub-rules have no Excel cell and no open owner:
  - an array expression derived from a range (`ISEVEN(Data!C1:C5+0)`): ours lifts it like an array
    constant (the SPEC-0011 `ARRAYS_1` entry, `Model::lifted_parity` in
    crates/rustsheet-engine/src/functions/information/mod.rs), so it spills;
  - legacy formulas: T-1774 moved ISEVEN/ISODD onto the `LiftArrays` rule probed for EDATE and
    YEARFRAC (probes/scalar-lift-shapes-c E, F, H, Q): a bare range gets no `@` and is `#VALUE!`, a
    value operation over a range gets `@`, `{5}` gets no `@`. That is inferred by analogy, not
    probed for ISEVEN/ISODD.
- Opus: the probe must separate "array argument lifts" from "anything range-derived is refused" (the
  range refusal is unusual among IS* functions), and the legacy forms need the `legacy_formulas`
  case shape the harness loads through the implicit-intersection parse (T-1096 is still in progress
  for the engine adapter; if legacy cells cannot be scored yet, commit them pending and say so).
- Use the excel-probe skill (excel-graph oracle).

## Acceptance criteria
- [x] A committed probe case with an excel-graph sidecar covers, for ISEVEN and ISODD: `C1:C5+0`,
      `-C1:C5`, `C1:C5*1` over a 2-D range, `IF(TRUE,C1:C5)`, `INDEX(C1:C5,0,1)`, `TAKE(C1:C5,3)`,
      `(C1:C5)`, a defined name for the range, and a LET-bound range, with the same mixed data as
      is-functions-array-lift (numbers, text, TRUE, an error, an empty cell), placed both on a row
      inside and outside the range.
- [x] The same case holds the legacy forms of a bare range, `C1:C5+0` and `{5}` (or records why
      the harness cannot score them yet, with a follow-up task).
- [x] Every new cell passes, or each remaining failure is owned by a filed task named in
      information.md.
- [x] information.md "ISEVEN/ISODD over a range" drops the UNVERIFIED sentence for every probed form
      and cites the new cells.
- [x] No new known-failure lines in eval/baseline/probes.txt other than the new case's owned cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Magnitudes of 2^63 or more (settled, T-2165).
- Other IS* functions, which lift ranges already (information.md).

## Outcome
- New probe probes/iseven-isodd-range-derived-arrays (excel-graph, 2026-11-06), 192 scored cells.
  Data!E1:E7 holds 2, 3, "x", TRUE, #N/A, an empty cell and -7.5; F1:F7 serves the 2-D form. The
  dynamic forms sit on a row inside the range's rows (2) and one outside (40); the legacy forms
  sit on another sheet with FORMULATEXT below each.
- Dynamic: a value operation over a range is an array and lifts (`E1:E7+0`, `-E1:E7`,
  `E1:F7*1`). Every form that still yields a reference is one `#VALUE!` with no spill, like the
  bare range: `IF(TRUE,R)`, `INDEX(R,0,1)`, `TAKE(R,3)`, `(R)`, a defined name and a LET-bound
  range. The engine already matched all 152 dynamic cells.
- Legacy: a bare range gets no `@` and a value operation over a range gets `@`, as inferred. But an
  array constant gets `@` too, against the inference: `ISEVEN(@{5})`, `ISEVEN(@{2;3})`,
  `ISEVEN(-@{5})`. The extra cells show this holds for every Analysis ToolPak (`LiftArrays`)
  argument, not just ISEVEN's: EDATE, YEARFRAC, BIN2DEC, QUOTIENT. The engine showed `ISEVEN({5})`
  and `@ISEVEN({2;3})`; the values matched either way.
- Fix: `array_constant_context` (static_analysis/lifted.rs) gives a `LiftArrays` argument the
  scalar signature and value position for its array constants. Both the `@` insertion (import)
  and its removal (export) use it. Ranges in those arguments keep their own rule.
- Tests: `lift_arrays_arguments_intersect_array_constants` (test_implicit_intersection_export.rs).
- information.md "ISEVEN/ISODD over a range" now records the dynamic and legacy forms from the
  probe and drops its UNVERIFIED sentences. arrays.md's ATP legacy entry notes the array-constant
  `@`.
- Eval delta: +1 probes case (192 cells, all pass); known failures 6606 -> 6606, 0 regressions in
  any suite.
