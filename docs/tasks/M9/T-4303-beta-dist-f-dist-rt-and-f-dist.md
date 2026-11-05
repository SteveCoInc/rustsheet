---
id: T-4303
title: "BETA.DIST, F.DIST.RT and F.DIST last digits at df2 5E9 to 1E10 on T-4030's df1-boundary probe (167 Phase 1b residue cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Each cell listed below is within half a unit of the 15th significant digit of Excel's value, or is listed in the
Outcome with Excel's value, ours, the 50-digit math and the reason Excel's digits cannot be derived.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 642d7f82, every failing cell checked against the open tasks with
  `scripts/exit_owners.py`'s ownership rule): T-4030 (done) drew probes-numeric/t4030-f-dist-rt-df1-boundary (235
  cells, excel-graph) to locate the `#NUM!` band of F.DIST.RT at df2 1E10. T-4044 (done) took the band itself. The
  numbers left over have no owner: 126 BETA.DIST, 36 F.DIST.RT and 5 F.DIST cells. None is visibly off.
- All are in the incomplete-beta family, so decision 0267 item 2 counts them as Phase 1b residue: they are first wrong
  at the 14th or 15th digit. Decision 0267 item 4 parks such family work, hence `defer: phase-1b`.
- Shapes: `BETA.DIST(x,df2/2,df1/2,TRUE)` with a from 5E8 to 5E9, b from 30 to 39.5 and x within 4E-8 of 1, and
  `F.DIST.RT(2,df1,1E10)`/`F.DIST(2,df1,1E10,TRUE)` at df1 61 to 76. Examples:
  - A100 `BETA.DIST(0.9999999850000002,5E9,37.5,TRUE)`: Excel 6.228844001972617E-07, ours 6.228844001972667E-07;
  - A116 `BETA.DIST(0.99999996,1.0E+9,30.0,TRUE)`: Excel 0.04322866333155803, ours 0.043228663331558496;
  - A1 `F.DIST.RT(2,61,1E10)`: Excel 5.855094527257988E-06, ours 5.855094527257928E-06;
  - A10 `F.DIST(2,65,1E10,TRUE)`: Excel 0.9999969188678635, ours 0.9999969188678633.
- Excel is the noisy one on BETA.DIST here. Against the case's `.math.json` (rustsheet-accuracy, 50 digits on the
  double inputs), ours is closer on 125 of the 126 cells. The median distance from the math is 1.6 ulp for ours and
  74 ulp for Excel; the largest is 43 ulp for ours and 468 for Excel. The F.DIST cells are within 1 ulp of the math
  on both sides. The 36 F.DIST.RT cells have no math value: `skipped` in the sidecar.
- Kernel: `bratio` dispatch on `fisher.rs` `f_beta_arguments` at a huge a with b near 30 to 40, the region where
  T-4030 found the `#NUM!` edge move with a (statistics.md, the T-4030 table). Decision 0188 had already measured
  that a correctly rounded kernel stays off Excel there. Work it only after Phase 1 exits.
- Cells:
- probes-numeric/t4030-f-dist-rt-df1-boundary.case.json (167): Calc!A1, A3, A4, A5, A7, A9, A10, A11, A12, A13, A15, A17, A19, A20, A21, A23, A25, A27, A29, A30, A31, A37, A41, A42, A43, A44, A45, A46, A47, A48, A49, A50, A51, A52, A53, A54, A55, A56, A57, A58, A59, A85, A86, A87, A88, A89, A90, A91, A92, A93, A94, A95, A96, A97, A98, A99, A100, A101, A116, A117, A118, A119, A121, A122, A123, A124, A125, A126, A127, A128, A129, A130, A131, A132, A133, A134, A135, A136, A137, A138, A139, A140, A141, A142, A143, A144, A145, A146, A147, A148, A149, A150, A151, A152, A153, A154, A155, A156, A157, A158, A159, A160, A161, A162, A163, A164, A165, A166, A167, A168, A169, A170, A171, A172, A173, A174, A176, A177, A178, A179, A180, A181, A182, A183, A184, A185, A186, A187, A188, A189, A190, A191, A192, A193, A194, A196, A197, A198, A199, A200, A201, A202, A203, A204, A205, A206, A207, A208, A209, A210, A211, A212, A216, A217, A218, A219, A220, A221, A222, A223, A224, A225, A226, A227, A228, A229, A230

## Acceptance criteria
- [ ] Each listed cell is within half a unit of Excel's 15th significant digit, or is listed in the Outcome with the
      three values and the reason (Excel off the math by more than ours, with no derivable algorithm).
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring;
      - a fresh sweep after the freeze;
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] These known-failure lines disappear from eval/baseline/probes-numeric.txt, or are explained in the Outcome,
      with no new ones (`make eval-baseline` refuses regressions). The `#NUM!` cells of the probe stay matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` band edge (T-4044, done) and the other F/beta residue tasks (T-3592, T-2221, T-3198, T-3323).
