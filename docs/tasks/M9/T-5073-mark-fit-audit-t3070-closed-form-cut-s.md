---
id: T-5073
title: "Mark fit-audit-t3070-closed-form-cut's 34 GAMMA.INV closed-form-branch cells at shape 1.3E10 to 2.3E10 as R3 points (decision 0337; Excel 2.66E-4 off the math, ours on it)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The 34 closed-form-branch cells of probes-numeric/fit-audit-t3070-closed-form-cut carry R3 marks in
eval/accuracy/math-over-excel.tsv. Then they land in the `r3_math` bucket instead of counting as visibly off in the M9
probe row and in GAMMA.INV's Phase 1 rate. The engine already returns the math there, by design (decision 0337).

## Context
- Gap triage 2026-11-08 (`make eval` on origin/main 6f0c3d8e5, excel-graph). The cells are Tier A under decision 0267:
  the first wrong digit is the 4th. They are counted under decision 0188 R1(c) because Excel is more than 1e-9 off
  the math and the cells have no R3 mark. Their only owner is T-3633, which scores the closed-form cut's FITTED
  bracket on this case. A fit task is the wrong owner for an R3 point (decision 0337: "do not put Excel's start back
  there"), so the marks are this task's. T-3633 keeps the case for its cut scoring: the marks change no engine value
  and no Excel value that scoring reads.
- They sit inside decision 0337's committed R3 case: statistics.md "GAMMA.INV at a huge shape, closed-form branch:
  Excel is about 2.7e-4 off, RustSheet returns the math (MATH-OVER-EXCEL)". The shapes are 1.3E10, 1.5E10, 1.7E10,
  1.95E10 and 2.3E10, all at or above 1E9. Every one has Excel / ours = 1.000266, the branch's signature, and the
  case's committed 50-digit math sidecar (fit-audit-t3070-closed-form-cut.math.json) equals ours to about 1E-16
  relative. For example, S!A1 `GAMMA.INV(1-2.305E-15,13000000000,1)`: Excel 13004349270.410213, ours
  13000893425.676235, math 13000893425.676236 (Excel 2.658E-4 off). S!A48 `GAMMA.INV(0.04902,23000000000,1)`: Excel
  23005862996.462105, ours and math 22999749093.71578.
- Decision 0337 marked only the probes cases (t3070-gamma-inv-band, t2982, t3350, incomplete-gamma-cap). This
  probes-numeric fit-audit case was missed. T-5015 does the same job for the t3409 sweeps. This is a sibling, not a
  duplicate: it covers different cells.
- The other 25 cells of the case (A7-A11, A19-A23, A31-A35, A43-A47, A55-A59) are on the true-quantile side of the cut
  and already match Excel. They get no mark.
- The `make eval` scan found no other unmarked branch cells. It searched every counted cell with Excel / ours between
  1.00025 and 1.00028. Every other one is owned by T-5015 (t3409 sweeps), T-4205 or T-4215 (below shape 1E9, where
  Excel's start stays the target), or T-4892 (the five gen-m9 multi-call cells with no math value).
- eval/accuracy/math-over-excel.tsv is a protected path (`scripts/accuracy_governance.py` `PROTECTED_FILES`), hence
  opus. The R3 case and its owner ruling exist already (decisions 0188 R3 case 3 and 0337), so no new ruling is
  needed. If `check_governance.py` asks for a decision record in the PR, add a short one that cites 0337 and lists
  the 34 cells.

Cells this task owns for their R3 mark (the fit and cut work on the case stays with T-3633):
- probes-numeric/fit-audit-t3070-closed-form-cut.case.json (34): S!A1, A2, A3, A4, A5, A6, A12, A13, A14, A15, A16, A17, A18, A24, A25, A26, A27, A28, A29, A30, A36, A37, A38, A39, A40, A41, A42, A48, A49, A50, A51, A52, A53, A54

## Acceptance criteria
- [ ] Each of the 34 cells has a line in eval/accuracy/math-over-excel.tsv
      (`probes-numeric/fit-audit-t3070-closed-form-cut.case.json!S!A<n>`), pointing at the statistics.md
      closed-form-branch entry's anchor that the existing t3070-gamma-inv-band lines use. Each passes decision 0337's
      per-cell check: Excel more than 1e-4 relative off the committed math value, the engine within 1e-13 of it, and
      the cell on the underflow branch. A cell that fails the check gets no mark. The Outcome lists it with Excel,
      ours and the math.
- [ ] The statistics.md entry's Evidence line names this case and the marked count.
- [ ] After `make eval`, the 34 cells are in the `r3_math` bucket, not `r1c_off_cells`. The M9 probe row's R1(c)
      count drops by the number marked. `make eval-baseline` records any visible-parity lock lines this frees. No
      engine value changes, and `eval/baseline/` gains no new line.
- [ ] `make check` green; reviewers approve (excel-fidelity included).

## Out of scope
- Engine changes, and the closed-form cut's FITTED bracket (T-3633).
- Cells below shape 1E9 (T-4205, T-4215), the t3409 sweeps (T-5015), and the gen-m9 multi-call cells (T-4892).
