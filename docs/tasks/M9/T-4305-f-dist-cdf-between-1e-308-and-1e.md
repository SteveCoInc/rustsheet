---
id: T-4305
title: "F.DIST cdf between 1E-308 and 1E-300 at df1 316 to 377: the 36 one-to-two-ulp cells of T-4026's subnormal probe"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Each F.DIST cell listed below matches Excel bit for bit, or is listed in the Outcome with Excel's value and ours.

## Context
- Found by T-4026 (done), Outcome: probes/t4026-f-cdf-subnormal (210 cells, excel-graph, 2026-11-05; generator
  docs/research/t4026/mkprobe.py) put the subnormal cut (DBL_MIN on the final cdf, statistics.md "F.DIST's cdf
  flushes a subnormal result to 0 (T-4026)") into the engine. It left 36 cells 1 to 2 ulp off and filed no owner.
  Gap triage 2026-11-05 (origin/main 642d7f82) confirms no open task names them.
- All 36 are `F.DIST(x,df1,df2,TRUE)` with results between 1E-308 and 1E-300: df1 316 at df2 1 (rows 7-23), and
  df1 377 at df2 1.123 (rows 43-104). Example: A10 `F.DIST(3.85366344071E-5,316,1,TRUE)` is Excel
  2.199991206165814E-305 and ours 2.1999912061658146E-305. None is visibly off, hence `defer: phase-1b`.
- The math sidecar does not settle it. Excel and ours both sit about 210 ulp (median) from the case's `.math.json`.
  ln F is about 158 ln x, so the cdf is very sensitive to x: the sidecar is likely evaluated at the decimal literal,
  not at the double. 11 cells have no math value. A rerun at the double x is the first step.
- Kernel: bpser at a = df1/2 158 or 188.5 with a tiny x0, `fisher.rs` `f_dist_of` then `bratio::dispatch`. The
  prefactor `x^a` near DBL_MIN is where Excel's and our evaluation order may differ (T-3192, T-3886: bpser items).
- Cells:
- probes/t4026-f-cdf-subnormal.case.json (36): Calc!A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A43, A44, A48, A49, A50, A52, A55, A56, A74, A75, A78, A81, A82, A85, A93, A98, A99, A100, A104

## Acceptance criteria
- [ ] Each listed cell matches Excel bit for bit, or is listed in the Outcome with Excel's value, ours, the math at
      the double input, and the reason.
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring;
      - a fresh sweep after the freeze;
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] These known-failure lines disappear from eval/baseline/probes.txt with no new ones (`make eval-baseline`
      refuses regressions). The 6 flushed-zero cells and their `=(..)=0` companions stay matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.DIST.RT, FDIST and BETA.DIST at a subnormal result (T-4282).
