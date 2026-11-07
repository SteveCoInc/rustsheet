---
id: T-4887
title: "F density with both df above 1E8: engine 3E-12 to 2E-10 off Excel's own density (T-4857 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`F.DIST(x,d1,d2,FALSE)` with both df at 1E8 or more matches Excel within 13 digits on the T-4857 cells, or the
residue is measured and documented.

## Context
- The density is Excel's own chain M-final (`f_density.rs`, statistics.md "F density is not the beta density ...
  (T-2513)"), fitted on df up to 1000 with T-3478's check to 1E10 for one df only.
- T-4857's pairwise rows (probes-numeric/t4857-f-classes, excel-graph 2026-11-07) reached the both-huge corner the
  density research never probed. Excel is itself 2E-9 to 1.1E-6 off the 50-digit density there (its K constant and
  the -h·L(r) term at h near 1E10), and the engine, which models that error, is 3E-12 to 1.8E-10 off Excel:
  `F.DIST(0.999969415022928,1E10,1E10,FALSE)` 1.8E-10 (A882; Excel 1.1E-6 off the math),
  `F.DIST(1.00104510205569,154482013.76,193693923.06,FALSE)` 7.0E-12 (A838), `F.DIST(0.999645837451614,120921417,
  3587386253,FALSE)` 3.1E-12 (A839).
- Code: `f_density.rs` `excel_f_density`, `constant`, `constant_offset`.

Cells (probes-numeric/t4857-f-classes): A838:A839, A882

## Acceptance criteria
- [ ] The three cells within 13 digits of Excel, or a time-boxed block with the measured residue; no density cell
      on file regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The density at one huge df (T-3478's region).
