---
id: T-4857
title: "F family coverage: equivalence-class probe over argument kinds, ranges and kernel branch boundaries (owner request)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The F-distribution family (F.DIST cdf and pdf, F.DIST.RT, FDIST, F.INV, F.INV.RT, FINV, and the argument kinds of
F.TEST and FTEST) has Excel evidence in every equivalence class of its arguments and in every pair of numeric
classes, with probe points on both sides of, and at, each branch edge of the incomplete-beta kernel. The evidence
says where the engine's agreement with Excel rests on extrapolation, and any gap it finds has an owner task.

## Context
- Owner request, 2026-11-07, verbatim: "do some thorough analysis of the arguments and their types and their
  ranges, and thinking about the general shape of the functions involved here, and then come up with a probe that
  tries really hard to do a nice job of getting a full set of equivalence classes across all of the different kind
  of interesting ranges of the input arguments, so that we can have more confidence that we're capturing all of the
  interesting points to spot areas of discontinuities and nonlinearities that might otherwise easily escape our
  extrapolation confidence."
- The worry: the family's evidence is clustered in the regions earlier research tasks probed (TOMS 708 branch
  studies, the huge-df2 sweeps), so the engine may be tuned to those points and extrapolate across regions where
  Excel switches algorithm, jumps, kinks or loses digits.
- Code: `crates/rustsheet-engine/src/functions/statistical/` (the F wrappers, `bratio`, `incomplete_beta.rs`,
  `large_shape_beta.rs`, the F density, the quantile searches). Semantics: `docs/excel-semantics/statistics.md`,
  the coercion docs.
- Neighbours: T-4434 sweeps df2 1E4..1E10 with df1 3..400 (this task references it and does not sweep that region
  densely); T-1871 fixes F.INV (this task files no duplicate for its cells).
- Probe freeze: decision 0267 item 5. The numeric case is named as an exception by this task's decision record,
  citing the owner's request.

## Acceptance criteria
- [ ] docs/research/t4857/ANALYSIS.md maps every argument kind, every domain and range edge and every kernel branch
      boundary of the family to equivalence classes, each grounded in code or evidence, and describes the functions'
      shape (nonlinearities, cancellation, underflow and overflow, the x <-> 1/x symmetry).
- [ ] A committed coverage-audit script bins every existing Excel cell of the family (eval/suites and
      eval/accuracy/tables) into the classes and their pairs, and reports the empty and thin ones.
- [ ] A seeded generator under docs/research/t4857/ writes a kinds probe case and a numeric probe case (about 2,000
      to 4,000 cells in all): every class several times, every numeric class pair, boundary triples at each branch
      edge, dense 1-D sweeps across the riskiest edges, symmetry pairs, and subnormal companions. Both are filled by
      excel-graph and their oracle and math sidecars committed.
- [ ] A committed scoring script reports Excel vs math, engine vs Excel and engine vs math per class and per pair,
      type mismatches, regime changes along the sweeps, and the low-prior-evidence classes where the engine drifts
      from Excel. The summary tables are in this task's Outcome.
- [ ] Every new Excel quirk is in docs/excel-semantics/statistics.md; every real engine gap has a filed task with
      cell lists; new failing cells enter the baseline only through `make eval-baseline`; `make eval-math-check`
      prints current.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (follow-up tasks own them).
- Dense coverage of T-4434's region (df2 1E4..1E10 with df1 3..400).
- F.TEST and FTEST value accuracy over data arrays (only their argument kinds and shapes are probed).
