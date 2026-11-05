---
id: T-4215
title: "Gamma quantiles at shape 5E5 to 5E9: the 196 unowned 1-to-4-ulp cells of probes/t3350-chisq-inv-huge-df and t3350-gamma-inv-band-lower-edge (Phase 1b)"
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
After Phase 1, the gamma quantiles (CHISQ.INV, CHISQ.INV.RT, CHIINV, GAMMA.INV) at a shape of 5E5 to 5E9 give Excel's
last digits on the listed probe cells. That is, the engine's inverse search ends on the double Excel's search ends on.
If that is not possible, the cells are recorded as being under the reproducibility floor.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main b450050f8). Every failing cell of these two gated probes cases that
  no open task lists. All of them are within 15 digits, at 1 to 4 ulp (most at 1 ulp), so they are not visibly off
  (decision 0114: `defer: phase-1b`).
  - CHISQ.INV 86 cells and CHISQ.INV.RT 70, at df 1E6 to 1E11 (shape df/2).
  - GAMMA.INV 32 cells at shapes 1E9 to 1E10, p 1E-40 to 1E-50 (the band's lower edge).
  - CHIINV 8 cells at df 2E7 to 1E9.
  - Nearly all are 1 ulp. The few others are 2 to 4 ulp.
  - Example: `CHISQ.INV(0.1,5E8)`: Excel 499959474.209413, ours 499959474.20941293 (1 ulp).
  - Example: `GAMMA.INV(1E-40,1E9,1)`: Excel 999579130.4319948, ours 999579130.4319949.
- Every cell is within decision 0188 R4.4's floor, which is 4 ulp web against desktop
  (docs/research/oracle-calibration.md). Before any fit, check whether these bits are stable between fills. T-3992
  measured fill-to-fill stability on gamma quantiles. Cells under the floor are not targets: record them and stop.
- Related owners, not duplicated:
  - T-3272: last digits from a shape of 1E9. Its listed cells are t3350-chisq-inv-huge-df A301:A351, the t3070 band
    and t2982. The cells here at shape >= 1E9 are in its scope but not in its list. Fold them into T-3272 if it is
    worked first.
  - T-4205: CHISQ.INV.RT(1E-300,df) below 1E9.
  - T-3396: the GAMMA.INV band edge `#NUM!`.
  - T-3294: the deep lower tail at shape 20 and above.
- Excel's search model: T-3409 (done), statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"; code
  crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/huge_shape.rs.

Cells (known-failure lines of eval/baseline/probes.txt):
- probes/t3350-chisq-inv-huge-df.case.json (158): S!A3, A4, A5, A6, A7, A10, A11, A13, A15, A17, A18, A19, A22, A26, A27, A28, A32, A36, A39, A40, A43, A44, A47, A50, A51, A52, A53, A54, A57, A59, A60, A64, A65, A66, A67, A68, A69, A72, A74, A75, A76, A77, A79, A85, A86, A88, A89, A90, A91, A93, A94, A96, A98, A99, A100, A102, A103, A105, A106, A110, A111, A112, A115, A119, A124, A125, A126, A127, A130, A131, A132, A135, A138, A139, A141, A143, A144, A148, A150, A151, A181, A182, A187, A188, A189, A190, A195, A196, A199, A200, A201, A204, A207, A208, A210, A216, A220, A222, A223, A227, A230, A231, A232, A233, A236, A237, A239, A242, A243, A244, A245, A246, A247, A248, A249, A250, A251, A252, A256, A259, A261, A262, A263, A267, A268, A269, A271, A272, A273, A275, A276, A278, A279, A280, A281, A283, A284, A287, A290, A291, A292, A295, A298, A299, A352, A354, A355, A356, A359, A360, A361, A363, A364, A366, A367, A368, A370, A372
- probes/t3350-gamma-inv-band-lower-edge.case.json (38): S!A1, A2, A3, A4, A6, A14, A18, A19, A20, A22, A27, A30, A31, A40, A41, A42, A43, A55, A56, A58, A68, A70, A71, A81, A84, A93, A94, A95, A97, A106, A119, A120, A131, A140, A143, A146, A149, A152

## Cells added by gap triage (2026-11-04)
`make eval` on origin/main 3c8d8c7d3. These GAMMA.INV cells of the T-3530 fit audit, at the band's lower edge, are 1 to
2 ulp off, and no open task lists them. They are within 15 digits (Phase 1b). Shapes are 5E9, 7E9 and 1E10, and p is
1E-34 to 1E-89. For example S!A39 `GAMMA.INV(1.25E-34,10000000000,1)`: Excel 9998778277.486937, ours 9998778277.486935.
The cells above 5E9 fall in T-3272's shape range too, and the note under "Related owners" applies.
- probes-numeric/fit-audit-t3070-band-lower.case.json (9): S!A26, A27, A32, A33, A35, A36, A39, A41, A42

Cells added by gap triage 2026-11-05 (`make eval` on origin/main 3c8d8c7d3, excel-graph evidence; the `owned` rule of
`scripts/exit_owners.py` over every failing cell against every open task): the same 1-to-3-ulp end of the inverse
search at a shape of 1E5 to 1E10, on a case no open task named. None is visibly off.
- probes-numeric/t3350-r4-chisq-inv-sweep.case.json (112; T-3350's R4 fresh sweep, df 1E5 to 1E10, CHISQ.INV 60 and
  CHISQ.INV.RT 52, all 1 or 2 ulp): Calc!A6, A7, A11, A21, A23, A24, A29, A33, A41, A42, A43, A44, A45, A46, A51, A53, A54, A55, A56, A57, A60, A61, A64, A66, A67, A68, A70, A71, A72, A73, A75, A76, A77, A78, A80, A81, A82, A83, A84, A86, A87, A88, A90, A91, A93, A94, A97, A98, A99, A102, A103, A105, A106, A107, A108, A110, A114, A118, A119, A120, A125, A143, A146, A154, A162, A163, A164, A166, A167, A171, A172, A175, A176, A178, A180, A183, A184, A186, A188, A190, A191, A192, A193, A194, A197, A199, A200, A201, A202, A203, A205, A207, A208, A209, A210, A211, A212, A213, A215, A217, A218, A219, A220, A221, A226, A227, A232, A233, A234, A235, A236, A237.
  E.g. Calc!A102 `CHISQ.INV(0.998817121141995,25597843)`: Excel 25619600.069028173, ours 25619600.06902817 (1 ulp).
  A122, A127 and A155 of this case stay with T-3272 and T-3364; its small-df cells are T-2695's.

## Acceptance criteria
- [ ] The listed known-failure lines disappear from eval/baseline/probes.txt with no new ones, and the 112
      t3350-r4-chisq-inv-sweep cells added on 2026-11-05 pass bit-exact (`make eval-show`). Or, for each cell left,
      the Outcome records its ulp and whether it is under the R4.4 floor (unstable between fills, or within 4 ulp web
      against desktop).
- [ ] Any fitted step carries the R4 guards (decision 0188, reviewer-excel-fidelity check 7): hold-out counts, a fresh
      sweep, the Gate A envelope result, a `FITTED(T-4215, n_cells, range)` tag and range, the reproducibility floor,
      and no keep-matched branch.
- [ ] `make check-fast` is green locally, CI is green, and reviewers approve.

## Out of scope
- The visibly-off and `#NUM!` cells of these cases (T-3272, T-3396, T-4205).
