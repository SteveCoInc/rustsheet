---
id: T-4304
title: "GAMMA.DIST large-shape last digits on the t3252 prefactor sweep and hold-outs outside T-4290's visibly-off cells (52 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4290]
defer: phase-1b
---

## Goal
Each GAMMA.DIST cell listed below matches Excel bit for bit, or is listed in the Outcome with Excel's value, ours and
the 50-digit math.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 642d7f82; every failing cell checked against the open tasks with
  `scripts/exit_owners.py`'s ownership rule). T-4290 (research, `campaign: visible`) owns the visibly-off cells of
  the t3252 prefactor probes. These 52 cells of the same probes are within 15 digits of Excel and no open task names
  them. They are 1 to 2 ulp: 43 NumericUlp and 9 NumericClose, all at the 16th or 17th digit. Decision 0188 R5 makes
  this Phase 1b, hence `defer: phase-1b`.
- Examples:
  - holdout-2 A1 `GAMMA.DIST(26.4,40,1,FALSE)`: Excel 0.004651507749641528, ours ...529;
  - holdout-2 A2 `GAMMA.DIST(26.4,40,1,TRUE)`: Excel 0.008119902132027311, ours ...313;
  - sweep A2 `GAMMA.DIST(121.4134,170.61,1,TRUE)`: Excel 1.464753581284843E-05, ours 1.4647535812848443E-05.
- Root cause (suspected): this is the large-shape (a >= 20) prefactor `x^a e^-x / Gamma(a)`, the same exponent T-4255
  and T-4290 study (statistics.md "Incomplete gamma: accurate kernel", the T-4255 paragraph). The prefactor's last
  digits are expected to follow once T-4290 identifies Excel's in-window exponent, hence the dependency. The probes
  have no `.math.json`; T-3925 (Gate A tables for GAMMA.DIST) adds the reference.
- Needs no new sweep to start: holdout-2 and holdout were drawn as hold-outs. A fit still owes R4's fresh sweep.
- Cells:
- probes/t3252-gamma-prefactor-holdout-2.case.json (3): Calc!A1, A2, A3
- probes/t3252-gamma-prefactor-holdout.case.json (1): Calc!A1
- probes/t3252-gamma-prefactor-sweep.case.json (48): Calc!A2, A4, A6, A8, A10, A11, A13, A14, A15, A16, A17, A18, A24, A25, A26, A27, A29, A31, A32, A33, A34, A36, A38, A40, A41, A42, A44, A47, A48, A49, A50, A51, A52, A56, A59, A61, A62, A65, A66, A69, A71, A72, A73, A75, A76, A77, A79, A80

## Acceptance criteria
- [ ] Each listed cell matches Excel bit for bit, or is listed in the Outcome with the three values and the reason.
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring;
      - a fresh sweep after the freeze;
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] These known-failure lines disappear from eval/baseline/probes.txt with no new ones (`make eval-baseline`
      refuses regressions), and T-4290's cells keep their state.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells of the same probes (T-4290) and of probes/t4255-gamma-window-exact-ratio.
