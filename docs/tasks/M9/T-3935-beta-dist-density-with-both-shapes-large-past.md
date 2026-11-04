---
id: T-3935
title: "BETA.DIST density with both shapes large past n = 256: the 1,168 unowned t3571-r5-bfrac-fresh pdf cells (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3244]
campaign: visible
defer: phase-1b
---

## Goal
`BETA.DIST(x,a,b,FALSE)` with both shapes large (a, b in [40, 5000], n = a + b - 2 mostly past 256) is within half a
unit of the 15th significant digit of Excel's value on the 1,168 pdf cells of T-3571's fresh bfrac sweep listed below,
with no new visibly-off cell anywhere.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; `scripts/exit_owners.py
  --all-off` plus a per-cell run of its `owned` rule over every failing cell of the report against every open task,
  deferred ones included, and the task files of every unmerged `origin/*` branch). No open task names this case's
  column B. T-3812 (done) used the case as its fresh sweep and recorded that the engine's density equals Excel's on
  only 32 of its 1,200 points; T-3680 owns only the four cdf cells with no math value.
- probes-numeric/t3571-r5-bfrac-fresh (ungated: probes-numeric has no baseline file), column B
  `=BETA.DIST(x,a,b,FALSE)` at log-uniform (a, b) in [40, 5000], one x per shape in TOMS 708's bfrac region. Failing
  1,168 of 1,200. By n = a + b - 2: 1,059 cells past 256 (678 visibly off), 108 in [90, 256] (38 visibly off), 1 below
  90. All are NumericClose or NumericUlp.
- Why visible and deferred: against the case's 80-digit math sidecar
  (eval/suites/probes-numeric/cases/t3571-r5-bfrac-fresh.math.json), Excel is a median 36 ulp (p90 181) from the
  math and the engine 29 ulp (p90 156); the engine is the closer one on 695 cells. Both are within 1e-9 of the math,
  so decision 0188 R1(c) counts these as Phase 1b evidence and R5 parks dense-sweep residue like this one. Excel is
  the inaccurate one, so the stage is `visible` (reproduce Excel's Loader form), not `accuracy`. Same convention as
  T-3244 and T-3474.
- Root cause (suspected): the x-dependent part of Excel's Loader saddle-point density (bd0 and the exponent), the
  residue T-3244 studies at n in [90, 256] and T-3220 below 90. Nobody owns both shapes large past n = 256: T-3250's
  large-exponent work is a = 1025 with b = 1.5. The miss grows with n, as T-3244 and T-3250 both record.
  Evidence and model: docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point";
  code `crates/rustsheet-engine/src/functions/statistical/loader.rs` (`excel_bd0`, `excel_bd0_series`,
  `excel_beta_density`).
- Examples (Excel vs ours), `make eval-show CASE=probes-numeric/t3571-r5-bfrac-fresh.case.json`:
  - Calc!B1 `=BETA.DIST(0.732672756053141,2500.0,700.0,FALSE)`: Excel 8.800124421426676e-08, ours
    8.800124421426905e-08 (visibly off).
  - Calc!B10 `=BETA.DIST(0.844587793975625,726.67,110.92,FALSE)`: Excel 5.130189357311101, ours 5.130189357311097.
  - Calc!B100 `=BETA.DIST(0.281085972859859,125.5,173.982,FALSE)`: Excel 3.642009788103464e-05, ours
    3.642009788103451e-05 (visibly off).
- The cdf column A of the same case is T-3936's, which depends on this task: T-3812 found the cdf is fl(brc·f) and
  that Excel's model reproduces 694 of 1,200 cdf cells bit-exact once it is given Excel's density.
- Fitting: Excel's points here are the fresh sweep of a frozen model (T-3571), so a fit tuned on them needs its own
  fresh sweep after the freeze. Shapes past n = 256 have no other cached probe: start from the T-2513 r3 loader
  probes and request a sweep through the research lane if the tune set is too thin.

Cells (gap triage 2026-11-03m; 1,168 cells, all BETA.DIST(.,FALSE)):
- probes-numeric/t3571-r5-bfrac-fresh.case.json (1168): Calc!B1:B87, B89:B189, B191:B194, B196:B286, B288:B289, B291:B320, B322:B326, B328:B375, B377:B401, B403:B428, B430:B469, B471:B518, B520:B530, B532:B540, B542:B548, B550:B570, B572:B578, B580:B674, B676:B699, B701:B783, B785:B888, B890:B911, B913:B937, B939:B945, B947:B954, B956:B1037, B1039:B1054, B1056:B1110, B1112:B1148, B1150:B1157, B1159:B1187, B1189:B1194, B1196:B1200

## Acceptance criteria
- [ ] The listed cells are within 15 digits of Excel's value, or each one left over is named in the Outcome with its
      reason and an owner.
- [ ] No new visibly-off cell in any suite; T-3244's, T-3220's and T-3250's cells keep their verdicts; the
      known-failure lines in eval/baseline/ only shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3935, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] statistics.md's Loader entry records the piece found past n = 256, with its hold-out score, or the negatives
      tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column A (the cdf): T-3936.
- n below 256 on the other probes (T-3244, T-3220); a = 1025 with small b (T-3250); min(a,b) <= 1.

## Visible-parity stage
Target (decision 0114): every cell above within half a unit of the 15th significant digit of Excel's value. Bit-level
residue goes to a follow-up with `defer: phase-1b`.
