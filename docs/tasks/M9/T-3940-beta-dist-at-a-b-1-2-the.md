---
id: T-3940
title: "BETA.DIST at a = b = 1/2: the 61 one-ulp cells of T-3192's fresh sweep t3192-beta-half-half-3 (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3886]
defer: phase-1b
---

## Goal
`BETA.DIST(x,0.5,0.5,TRUE)` returns Excel's bits on the 61 cells of T-3192's fresh sweep that the direct arctangent
form misses, or the Outcome records them as the reflection's evidence.

## Context
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; per-cell `owned` run over every
  failing cell against every open task, deferred ones included, and every unmerged branch's task files). T-3192
  (done) drew probes-numeric/t3192-beta-half-half-3 as its R4.2 fresh sweep after freezing the direct form: 239 of 300
  bit-exact, every cell within 2 ulp. T-3886 owns the open a = b = 1/2 items (Excel's switch to a reflection above x
  near 0.9, the x = 0.5 cell, and the misses of sweeps 1 and 2) but does not name this third sweep.
- Column B `=BETA.DIST(A<n>,0.5,0.5,TRUE)`: 61 failing (all NumericUlp, 1-2 ulp, within 15 digits). Against the
  80-digit math sidecar Excel is a median 0 ulp (p90 1) and the engine 1 (p90 1), so Excel is close to correctly
  rounded here. E.g. Calc!B28: Excel 2.4397943825683136e-05, ours 2.439794382568314e-05; Calc!B109: Excel
  0.31763280936696203, ours 0.3176328093669621.
- Evidence and model: docs/excel-semantics/statistics.md "BETA.DIST at a = b = 1/2 is an arctangent closed form,
  reflected above x = 0.9"; docs/research/t3192 (`score_half.py`). Code: `incomplete_beta::half_half_cdf`.
- Depends on T-3886: if its reflection lands (with the R4.4 check or an owner ruling), re-score these cells first.
  These cells were never tuned on, so they stay hold-out for any form chosen there. The misses are below decision
  0112's floor.

Cells (gap triage 2026-11-03m; 61 cells):
- probes-numeric/t3192-beta-half-half-3.case.json (61): Calc!B28, B109, B112, B116:B117, B131:B132, B136, B162:B163, B165, B196, B199, B201:B202, B206, B208, B213:B214, B219, B221:B225, B227:B228, B233, B238, B241:B242, B245, B249:B255, B259, B261:B263, B265:B266, B269:B271, B275, B278, B281, B283, B287:B289, B291:B292, B295:B296, B298:B299

## Acceptance criteria
- [ ] Each listed cell passes bit-exact, or the Outcome lists the ones still open with the form tried.
- [ ] No BETA.DIST, BETADIST or T.DIST cell of any suite regresses; the known-failure lines in eval/baseline/ only
      shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3940, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3886's items (bpser at b0 >= 8, T-2505's moved cells, the shape-of-1 cells, x = 0.5).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
