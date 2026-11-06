---
id: T-4580
title: "Gate A: T.DIST (cumulative and density) uses the evidenced huge-x #NUM! cut rule (HugeXCut)"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
T.DIST's Gate A definition encodes Excel's huge-x `#NUM!` as T.DIST.RT, T.DIST.2T and TDIST do. Each of its
committed tables keeps 0 differences under `verify`.

## Context
- T-4102 (Excel fidelity review) added `student_t.HugeXCut` (tools/src/rustsheet_tools/accuracy/defs/student_t.py).
  - From the smallest evidenced `#NUM!` |x| on, x is outside the domain.
  - Strictly between the largest evidenced number and that `#NUM!`, the point has no value (NoValueError, unprobed).
  - The brackets come from every Excel cell: the tables, the probes and gen-m9.
- T.DIST on main still gives a math number at a huge x where Excel is `#NUM!`. statistics.md, "Scalar distributions at
  extreme arguments, part 2" and "T.DIST's density is #NUM! at a huge x":
  - Cumulative, negative x: 0 at x -4.0E+136 and -2.1E+119 (accuracy/T.DIST/s1 A90, A96); `#NUM!` at -2.1E+217 and
    -4.1E+259 (A103, A91).
  - Cumulative, positive x: only `#NUM!` cells, at 5.4E+186 and 4.4E+221 (A86, A98).
  - Density: `#NUM!` from 4.2E+220 for a positive x (e-T-3425 A116) and at -4.6E+224 (e-T-3425 A103). It is 0 at
    9.9E+137 (s1 A97) and, for a negative x, up to |x| 7.6E+87.
- The brackets are per sign and per `cumulative`. Re-derive them from every cell; do not assume the x² overflow point.
- The T.DIST batches (s1, e-T-3425, r4-T-2585) are already committed on main, so their math values cannot be
  rewritten in place: decision 0199 and SPEC-0029 require a changed value to be a new batch plus a decision record.
  Write that decision record, citing T-4102's brackets, before changing any committed T.DIST math sidecar.
  Only number cells may move to a domain error.
- The density known failures owned by T-3554 (engine 0, Excel `#NUM!`) stay error mismatches. Their lines stay.

## Acceptance criteria
- [ ] T.DIST's definition uses `HugeXCut` for both `cumulative` values, with the brackets cited from the evidence.
- [ ] `rustsheet-accuracy verify T.DIST` reports 0 differences, and `rustsheet-eval accuracy` reports 0 failing and 0
      table defects.
- [ ] The probe sidecars are regenerated with `--function T.DIST`, and only number cells move.
- [ ] A decision record (decision 0199) authorizes the changed T.DIST math values, or they go in a new batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Matching the engine's density to Excel's `#NUM!` (T-3554).
