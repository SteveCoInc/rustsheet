---
id: T-3757
title: "Owners for 177 unowned last-digit cells of the F density r4-wide sweep (Phase 1b)"
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
Each cell listed below is bit-exact with Excel. Alternatively, a cell can move to a narrower task that names its
kernel branch, or be listed in the Outcome with Excel, ours, the 50-digit math, and the reason Excel's bits cannot be
derived.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Gap triage 2026-11-01 (`make eval` on origin/main 26e0bd00b): these failing Excel cells are named by no open task,
  deferred ones included. Every one is within half a unit of Excel's 15th significant digit, or is Phase 1b evidence
  that decision 0188 R1(c) leaves out (both values within 1e-9 of the 50-digit math). So they are last-digit work,
  and decision 0188 R5 parks them as Phase 1b (BETA.DIST and F.DIST densities and dense sweeps inside Gate A), hence
  `defer: phase-1b`. Same shape as T-3592, which owns the other cells of this case. Keep the two lists disjoint.
  This task was filed with a second group, 104 BETA.DIST cells of probes-numeric/t2464-w2f-unit-shape; T-3735 landed
  on main first and owns that case's last-digit cells, so they were removed from this task.
- **F density past its small-shape fit set (177).** probes-numeric/t3478-f-density-r4-wide is T-3478's R4.2 fresh
  sweep of the F density (300 points, 11 <= min(d) <= 15 with max(d) up to 1E+10, and min(d) <= 10 with max(d)
  11..1E+10). T-3478 kept the `SMALL_SHAPE_OFFSET` and Stirling-error constants there because they beat the correctly
  rounded density on every stratum (61 against 15 bit-exact). docs/excel-semantics/statistics.md "F density's
  small-shape constant past its fit set" has the table. Example: Calc!A1 `F.DIST(0.736448028695065,12,11,FALSE)`, Excel
  0.7850132428730837, ours ...835. Three cells differ in the 15th digit but are Phase 1b evidence (Calc!A157
  `F.DIST(0.0487253862028046,5,34,FALSE)`, Excel 0.07735696654827515, ours ...509). Related model work: T-3158,
  T-3173, T-3174, and T-3592 (45 cells of the same case).
- Cells (`make eval-show CASE=<case>`):
  - probes-numeric/t3478-f-density-r4-wide.case.json (177): Calc!A1:A3, A6, A8:A12, A14, A17, A19, A21:A24, A27, A33,
    A35:A37, A39:A41, A45, A47, A49:A50, A55:A56, A58, A61, A65:A74, A76:A77, A79, A86:A89, A92:A93, A100:A103,
    A105:A106, A108, A111, A113, A115:A118, A121, A123, A126, A130, A132:A133, A135, A137:A143, A145, A147:A152, A157,
    A159:A160, A164:A165, A167, A169:A173, A175:A177, A181:A183, A186, A191:A194, A196, A198:A201, A203:A204,
    A206:A210, A213:A215, A217:A218, A220:A221, A225:A238, A240:A245, A247:A248, A250, A253, A255:A256, A258:A260,
    A265:A266, A269:A271, A276:A277, A279, A281, A283, A285, A287:A291, A293, A295:A299
- Code: crates/rustsheet-engine/src/functions/statistical/f_density.rs, fisher.rs.

## Acceptance criteria
- [ ] Every cell above is bit-exact, moved to a named narrower open task (listed there), or listed in the Outcome with
      Excel, ours, the 50-digit math and the reason. probes-numeric is ungated (no baseline file), so the counts
      before and after come from the report. No baseline under eval/baseline/ gains a line.
- [ ] Any kernel change carries decision 0188's R4 guards (reviewer-excel-fidelity check 7): hold-out counts for the
      chosen and the correctly rounded candidate on a split fixed before scoring; a fresh sweep drawn after the freeze;
      the Gate A envelope result on it; a `FITTED(T-3757, n_cells, range)` tag and range on fitted constants, with
      accurate math outside; nothing fitted below the reproducibility floor; no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells T-3592, T-3479 and T-3527 own. Visibly-off F.DIST cells outside this case. The t2464-w2f-unit-shape BETA.DIST cells (T-3735).

`defer: phase-1b`: every cell here is within Excel's 15 digits or is R1(c) Phase 1b evidence (decision 0188 R5).
