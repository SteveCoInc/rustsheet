---
id: T-5084
title: "Holding owner for the visibly-off probe cells no open task owns (main b84a7256): 85 cells"
milestone: M7
lane: quality
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every visibly-off probe cell has an open owner, so the Phase 1 owners criterion (decision 0187) reads them as owned.
This task holds the cells that no open task owned at main b84a7256 (2026-11-08). It is a low-priority catch-all
(owner's request), not a plan to fix them one by one.

## Context
- Found by the fresh full eval at main b84a7256 (`scripts/exit_owners.py --all-off` and `--milestone M7/M9`, plus the
  t4974 probes' cells that are not within 12 significant digits of Excel and that no open task names).
- What the cells are:
  - Decision 0329 superseded October cells (accepted known failures, kept by design; no fix wanted): gen-m9/e85d A35,
    t2436/e85d A31 and A46, t2592 A6:A8, t2594 A26:A29, t2625 A13, A17, A20, A25, A26. Their owner T-4736 is done.
  - New probe cells whose probe tasks are done:
    - t4901-mround-blank-let-lambda-roundings (T-4901) and t5067-written-empty-branch-wrappers (T-5067). The written-
      empty-branch read is T-4912's (M5), so this task records the split from T-4912 and holds the M7 exit cells.
    - The t4974 probes: BETA.INV and F.INV at p = 1E-300, BETA.INV's DBL_MIN residual, and GAMMA.DIST's density at
      shape 170.5 and 171 near x = 1. There Excel is 0, ours a subnormal-scale number.
- Cells:
  - gen-m9/e85daedce5c2675e.case.json (1): Calc!A35
  - probes/t2436/e85daedce5c2675e.case.json (2): Calc!A31, Calc!A46
  - probes/t2592-nested-groupby-cells.case.json (3): P!A6, P!A7, P!A8
  - probes/t2594-nested-aggregates.case.json (4): P!A26, P!A27, P!A28, P!A29
  - probes/t2625-nested-consumers.case.json (5): P!A13, P!A17, P!A20, P!A25, P!A26
  - probes/t4901-mround-blank-let-lambda-roundings.case.json (7): Calc!A3, Calc!A10, Calc!A12, Calc!A13, Calc!A14, Calc!A15, Calc!A16
  - probes/t4974-beta-f-inv-endpoint-saturation.case.json (2): P!G5, P!H5
  - probes/t4974-beta-inv-dbl-min-residual.case.json (11): P!B1, P!D1, P!K2, P!K3, P!K4, P!K5, P!K6, P!K7, P!K9, P!K10, P!K11
  - probes/t4974-f-inv-tiny-tail-neighbourhood.case.json (6): P!A8, P!A9, P!B8, P!C7, P!D7, P!E9
  - probes/t4974-gamma-small-x-prefactor-boundary.case.json (27): P!P1, P!P2, P!P3, P!P4, P!P5, P!P6, P!P7, P!P10, P!P11, P!P12, P!P13, P!P14, P!P15, P!P16, P!R1, P!R2, P!R3, P!R4, P!R5, P!R6, P!R7, P!R10, P!R11, P!R12, P!R13, P!R14, P!R16
  - probes/t5067-written-empty-branch-wrappers.case.json (17): Calc!A1, Calc!A2, Calc!A3, Calc!A11, Calc!A12, Calc!A14, Calc!A15, Calc!A16, Calc!A20, Calc!A21, Calc!A22, Calc!A23, Calc!A24, Calc!A25, Calc!A26, Calc!A28, Calc!A29

## Acceptance criteria
- [ ] Each cell above is fixed by a task that owns its cause, moved to such a task, or (decision 0329 cells) stays
      here as an accepted known failure.

## Out of scope
- New work on the superseded decision 0329 cells (Excel's later fill is the target, decision 0329).
