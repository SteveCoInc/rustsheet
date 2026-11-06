---
id: T-4528
title: "ISEVEN and ISODD of an array expression over a range (C1:C5+0, -C1:C5, IF over a range) and their legacy-formula @ forms: probe and match"
milestone: M5
lane: functions-logic
status: todo
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
- [ ] A committed probe case with an excel-graph sidecar covers, for ISEVEN and ISODD: `C1:C5+0`,
      `-C1:C5`, `C1:C5*1` over a 2-D range, `IF(TRUE,C1:C5)`, `INDEX(C1:C5,0,1)`, `TAKE(C1:C5,3)`,
      `(C1:C5)`, a defined name for the range, and a LET-bound range, with the same mixed data as
      is-functions-array-lift (numbers, text, TRUE, an error, an empty cell), placed both on a row
      inside and outside the range.
- [ ] The same case holds the legacy forms of a bare range, `C1:C5+0` and `{5}` (or records why
      the harness cannot score them yet, with a follow-up task).
- [ ] Every new cell passes, or each remaining failure is owned by a filed task named in
      information.md.
- [ ] information.md "ISEVEN/ISODD over a range" drops the UNVERIFIED sentence for every probed form
      and cites the new cells.
- [ ] No new known-failure lines in eval/baseline/probes.txt other than the new case's owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Magnitudes of 2^63 or more (settled, T-2165).
- Other IS* functions, which lift ranges already (information.md).
