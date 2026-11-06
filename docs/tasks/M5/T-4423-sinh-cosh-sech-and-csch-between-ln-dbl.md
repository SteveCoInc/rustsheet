---
id: T-4423
title: "SINH, COSH, SECH and CSCH between ln(DBL_MAX) and 710.48: probe whether Excel's #NUM! starts where e^x overflows (the engine's cut) or where sinh itself does"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
SINH, COSH, SECH and CSCH match Excel at every |x| in (709.78, 710.48). That is the band where e^|x| overflows a double
but the true sinh and cosh do not. Today the engine is `#NUM!` (SINH, COSH) or 0 (SECH, CSCH) across the whole band,
from a cut that no Excel cell brackets.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 20b163db, excel-graph sidecars). `make unowned-off ALL=1` gives 0
  unowned visibly-off cells, so this triage read the rules the semantics docs mark unprobed. The functions-math lane had
  no ready task.
- numbers.md "SINH and COSH overflow is `#NUM!` at once; SECH there is 0 (T-2169)" says "the cut in (709.78, 710) is
  unprobed". Its evidence on either side:
  - SINH(709.78) is a number, and so are COSH(±709.78) and COSH(702.659631014027) (accuracy/COSH/s1, T-3670);
  - `SINH(710)` is `#NUM!` (gen-m5/25768de48ae7f2f9 Calc!A315), as are `SINH(-710)`, `COSH(±710)` and the other
    T-2169 cells;
  - `SECH(710)` is 0 (gen-m5/1962de140dae0111 Calc!A30, gen-m5/1b30e4d075b252ea Calc!A318);
  - `SECH(±709.78)` and `CSCH(±709.78)` are 0 (accuracy/SECH/s1 Calc!A83, A84; accuracy/CSCH/s1 Calc!A84, A72, A79).
- The rest of that entry and its COSH note ("the cut in (709.78, 710) stays unprobed for COSH and SECH as for SINH")
  say the same.
- Ours: `sinh` and `cosh` in crates/rustsheet-engine/src/functions/math_and_trigonometry/hyperbolic.rs compute
  `(x.exp() ∓ (-x).exp()) / 2`, so they overflow exactly where `exp` does: x = 709.782712893384 (ln DBL_MAX). The doc
  comment says "Excel's exact cut is unprobed, T-1944". A scratch case on a release build of 20b163db gives:
  - `SINH(709.782712893384)` 8.988465674311366E+307 and `SINH(709.79)` `#NUM!`;
  - `COSH(709.79)` `#NUM!`;
  - `SECH(709.79)` and `CSCH(709.79)` 0, and `SECH(709.5)` 1.4760296628025164E-308.
- The open question: does Excel round e^x to a double before the subtraction and halving (cut at 709.7827, as ours), or
  does it keep an 80-bit x87 value and only store the halved result (cut at asinh(DBL_MAX) = 710.4758600739439)? Or
  does it cut somewhere else? There is a hint in T-3727's cells (probes-numeric/t3616-imsin-imcos-positive-cut-probe
  N8, N16, N21, O8, O16, O21). Just below the band, Excel's COSH and SINH are one ulp above ours while EXP (column M)
  matches. That fits a form other than `exp(x)/2` rounded twice, such as `exp(x - ln 2)` or an 80-bit halving, and
  either of those would also move the cut. Read T-3727 before choosing a model. If Excel answers inside the band, the engine needs a kernel there that matches Excel's
  digits and stays inside Gate A's envelope. That is a formulation choice, hence opus.

## Probe
excel-probe skill, excel-graph, tag M5. Name the case probes/t4423-hyperbolic-overflow-band.
- SINH, COSH, SECH and CSCH, both signs, at:
  - the double just below ln(DBL_MAX), at ln(DBL_MAX) and the next three doubles;
  - 709.79, 709.8, 709.9, 710 (the control), 710.1, 710.2, 710.3, 710.4, 710.47;
  - asinh(DBL_MAX) = 710.4758600739439 and its neighbours;
  - 710.48.
- Then a second fill that bisects to 1E-12 around whatever cut the first fill shows.
- Wrap the reciprocals in `=0` and `ISNUMBER` companions, because a subnormal result reads as 0 on Graph (decision
  0042).

## Acceptance criteria
- [ ] probes/t4423-hyperbolic-overflow-band is committed with excel-graph oracle values, and the SINH and COSH cuts are
      bracketed by adjacent cells.
- [ ] Every cell of the case passes, with no new known failure in eval/baseline/. If Excel answers inside the band, the
      new kernel's model is stated in the PR as check 7 kind (b) or (c) of reviewer-excel-fidelity. If it is kind (c)
      or a fit, the Outcome shows these decision 0188 R4 guards:
      - hold-out counts;
      - a fresh sweep after the freeze;
      - the Gate A envelope result on it, every point outside listed;
      - a `FITTED(T-4423, n_cells, range)` tag for kind (a);
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] numbers.md "SINH and COSH overflow is `#NUM!` at once" and its COSH note replace "the cut ... is unprobed" with the
      bracketing cells. The doc comment of `sinh` drops "Excel's exact cut is unprobed".
- [ ] `rustsheet-eval accuracy` passes (the COSH, SINH, SECH and CSCH tables); `make check` green; reviewers approve.

## Out of scope
- EXP's own overflow edge (T-1944 and T-2203, done).
- IMSINH/IMCOSH past |x| 709 (engineering.md; T-2631).
- Last-digit residue below the cut: COSH and SINH one ulp off just above 709.7 (T-3727) and COSH at small x (T-2744),
  both `defer: phase-1b`.
