---
id: T-4297
title: "GAMMA.DIST large-shape prefactor: last ulps of 52 t3252 sweep and hold-out cells no task names (within 15 digits, after T-4290; Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4290]
defer: phase-1b
---

## Goal
GAMMA.DIST at shapes 30 to 300 near and below the mode (x/a 0.55 to 0.75) returns Excel's last bits on the 52 cells
below, all already within 15 significant digits of Excel, so that every failing cell of the T-3252 prefactor probes
has an owner. Below the T-3252 window that means the ratio form's last ulp; inside it, the in-window exponent
T-4290 reads.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main c3de9423b, excel-graph sidecars). `make unowned-off ALL=1` is
  0: every visibly-off cell has an open owner. These 52 are the GAMMA.DIST cells that fail bit-level with no open task
  naming them (`scripts/exit_owners.py` ownership rule applied to every failing cell, not only visibly-off ones).
  T-4290 names the visibly-off cells of the same cases. T-3252 and T-4255, which drew these cases, are done.
- What the cells show (expected and ours from `make eval-show CASE=probes/t3252-gamma-prefactor-sweep`):
  - Below the window (x/a 0.5594 to 0.6645), 1 to 2 ulp either way, density and cdf alike. Examples: A6
    `GAMMA.DIST(57.8074,99.86,1,TRUE)` Excel 3.304884010497024E-07, ours ...025E-07 (+1 ulp); A4
    `GAMMA.DIST(64.6229,104.49,1,TRUE)` Excel 3.1760343037236196E-06, ours ...619E-06 (-1 ulp); holdout-2 A1
    `GAMMA.DIST(26.4,40,1,FALSE)` Excel 0.004651507749641528, ours ...529 (+1 ulp). statistics.md "Incomplete gamma:
    accurate kernel" (T-3252 paragraph) says the ratio form is Excel's "to 0 to 2 ulp" there, and T-4255 found it
    Excel's to 2 ulp on exact x/a only when built on Excel's own LN(a/x). So the residue is an operation order in
    the ratio form, not a new branch. The engine takes the ratio form below `NEAR_MODE_BELOW` (0.334) at shapes 30
    to 500 (`crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs`).
  - In the switch bracket (0.65625, 0.671875] T-4255 found: A13 and A14 at x/a 0.6672, 13 ulp each.
  - Inside the window (x/a 0.688 to 0.746), 1 to 18 ulp: A36 `GAMMA.DIST(115.4846,163.5,1,TRUE)` Excel
    1.4866212188258928E-05, ours ...898E-05 (18 ulp low); A56 `GAMMA.DIST(82.9499,112.17,1,TRUE)` Excel
    0.00130220587791963, ours ...96275 (12 ulp low); A10 `GAMMA.DIST(51.4828,74.83,1,TRUE)` Excel
    0.0013249233649052757, ours ...776 (9 ulp high). Excel's model there is unknown (T-4255 Outcome: no tested form beats chance);
    T-4290 owns reading it. So this task depends on T-4290: it needs that research's sweep and model, and does not
    fit blind on these cells.
- Cells (all `numeric_ulp`/`numeric_close`, within 15 significant digits):
  - probes/t3252-gamma-prefactor-sweep.case.json (48): Calc!A2, A4, A6, A8, A10, A11, A13, A14, A15, A16, A17, A18, A24, A25, A26, A27, A29, A31, A32, A33, A34, A36, A38, A40, A41, A42, A44, A47, A48, A49, A50, A51, A52, A56, A59, A61, A62, A65, A66, A69, A71, A72, A73, A75, A76, A77, A79, A80
  - probes/t3252-gamma-prefactor-holdout.case.json (1): Calc!A1
  - probes/t3252-gamma-prefactor-holdout-2.case.json (3): Calc!A1, A2, A3
- Of the 48 sweep cells, 15 are inside the window (A2, A8, A10, A15, A16, A24, A31, A32, A36, A41, A42, A56, A65,
  A66, A69) and 33 at or below the switch. Decision 0267 does not apply: GAMMA.DIST is not in the incomplete-beta
  family, and none of these cells is visibly off.
- The sweep was the fresh sweep T-3252 drew after freezing `NEAR_MODE_BELOW`, so it must not become fit data for
  the same constant: any fit here takes a new fresh sweep (decision 0188 R4.2), and probes-numeric is frozen to
  "Decision 0267" tasks (decision 0267 item 5), so a new sweep goes under probes/.

## Acceptance criteria
- [ ] The 52 cells above are bit-exact, or each one left is listed in the Outcome with its ulp distance and the
      reason (Excel's noise, no form found), with no cell of the three cases or of
      probes/t3252-gamma-prefactor-window and probes/t4255-gamma-window-exact-ratio getting worse.
- [ ] Any fitted or ranked choice (a ratio-form operation order, a switch point, T-4290's in-window model) carries
      the R4 guards of reviewer-excel-fidelity check 7 (decision 0188): train and hold-out counts side by side
      (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and the correctly rounded one, with the
      split and margin stated before scoring; a fresh oracle-filled sweep drawn after the freeze, with its result;
      the Gate A envelope result of that sweep (count inside, every point outside listed); a `FITTED` tag with its
      fitted range on any fitted constant; the reproducibility floor; and no branch that only keeps already-matching
      cells (no keep-matched branch).
- [ ] No gated known-failure line is added (`make eval-baseline` refuses none); the GAMMA.DIST, CHISQ.DIST and
      POISSON.DIST cells of eval/baseline/probes.txt only shrink.
- [ ] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel" records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells of the same cases (T-4290 owns them) and the cdf far above the mean (T-4291).
- Shapes below 20 and above 500, and the 1.05a to 1.5a window above the mode, unless the same rule covers them.
