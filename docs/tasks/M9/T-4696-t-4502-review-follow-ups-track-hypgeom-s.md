---
id: T-4696
title: "T-4502 review follow-ups: track HYPGEOM's past-walk Excel targets (A95, A164), probe three empty-population shapes, one shared Dd alias"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4502]
---

## Goal
The Excel targets T-4502 (PR #3742) left behind are tracked again, three inferred empty-population shapes are probed,
and the statistical module has one double-double alias.

## Context
- Excel-fidelity review of T-4502: past the double-double walk the engine now computes R nmath's dhyper (the math), so
  Gate A points there pass on the math side only. Excel's own value differs at accuracy/HYPGEOM.DIST/s1 A95 (Excel 0,
  math and ours 8.1E-186) and at A164 (Excel 0.4538325650607541, ours 0.45383256506076325, 15th digit). With their
  known-failure lines gone, no gate, baseline or task tracks these Excel targets, though decision 0300 keeps Excel the
  target. statistics.md's T-4502 "Ours" paragraph says reproducing A95 "needs Excel's formulation (T-2481)", but T-2481
  covers small arguments inside the walk only. Record them in T-2481's context and acceptance (or here), so the
  Excel-side mismatch past the walk has an owner.
- The same entry infers, unprobed: `HYPGEOM.DIST(0,0,0,0,TRUE)` = 1, `HYPGEOMDIST(1,0,0,0)` = 0,
  `HYPGEOMDIST(0,0,1,0)` = `#NUM!`. Probe them (excel-probe skill, predictions first) and match.
- Design review of T-4502: a private `type Dd = (f64, f64)` exists in statistical/discrete.rs, loader.rs, hypegeom.rs,
  student_t.rs, and as `pub(super)` in bratio/bgrat.rs and incomplete_gamma/fraction.rs. Make it one `pub(super)` alias
  (e.g. statistical/mod.rs) and drop the copies.

## Acceptance criteria
- [ ] A95 and A164 (and any other past-walk Excel-side mismatch) are named in an open task's acceptance.
- [ ] The three empty-population shapes are probed (excel-graph) and pass.
- [ ] One `Dd` alias in the statistical module; duplication gate green.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reproducing Excel's own past-walk digits (that is T-2481's formulation question).
