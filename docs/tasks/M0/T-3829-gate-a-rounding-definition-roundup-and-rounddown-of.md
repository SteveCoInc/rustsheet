---
id: T-3829
title: "Gate A rounding definition: ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is subnormal give no math value (Excel 0, math 1)"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The Gate A rounding definition returns no math value for ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is
subnormal, instead of a value Excel never gives. Excel shows 0 there, under the VERIFIED rule.

## Context
- docs/excel-semantics/numbers.md "ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is subnormal" (T-2756,
  VERIFIED): Excel returns a subnormal, which shows as 0.
- T-3679's regenerated probes sidecars give math 1, 10, 0.01, 1E-300 or about 2.2E-308 for those cells. Examples:
  - probes/t2756-roundup-smallest-normal-threshold: `ROUNDUP(A1,0)` with A1 = 2^-1022 has math 1.0 and oracle 0.0;
  - about 100 cells in t2756 and t2715-roundup-near-smallest-normal, rows B, D and E and C11:C20.
- R1(c) is unaffected, because Phase 1b needs Excel within 1e-9 of the math, so these cells always count. But the
  docstring of tools/src/rustsheet_tools/accuracy/defs/rounding.py says every such point is within the 1e-300 floor,
  which these cells contradict. Any check that reads the math side alone would accept a semantic miss there.
- Found by the Excel-fidelity review of T-3679 (2026-11-02).

## Acceptance criteria
- [ ] The definition raises `NoValueError` when the 15-digit cut converts to a subnormal (ROUNDUP and ROUNDDOWN), or
      the cells are recorded as known math-vs-Excel semantic points; its docstring is corrected.
- [ ] The probes sidecars for t2756 and t2715 are regenerated and drop those cells; `make eval-baseline` records
      nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other rounding forms.
