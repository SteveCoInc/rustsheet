---
id: T-3939
title: "GAMMA.DIST and GAMMA.INV last digits on the fit-audit and t3316 probes (58 cells within 15 digits, Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3633]
defer: phase-1b
---

## Goal
GAMMA.DIST and GAMMA.INV return Excel's bits on the 58 last-digit cells of T-3530's fit-audit probes and T-3316's
subnormal-scale probe listed below, or the Outcome names each cell left with its kernel.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (gamma quantile) measured on t3311-gamma-inv-lower-tail-cdf, 0 of 899 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; per-cell `owned` run over every
  failing cell against every open task, deferred ones included, and every unmerged branch's task files). T-3633
  (score the provisional gamma FITTED cuts), T-3396, T-3294 and T-3409 name these fit-audit cases but list other
  cells (or only count them); T-3316 (done) filled its probe. These cells have no owner. All are within 15 digits.
- probes-numeric/fit-audit-t2852-shape-min (32): `=GAMMA.DIST(x,a,1,FALSE)` at shapes 0.11 to 0.6, 1-3 ulp. E.g.
  S!A9 `GAMMA.DIST(0.00731,0.2,1,FALSE)`: Excel 11.06111474547857, ours 11.061114745478564; S!A22
  `GAMMA.DIST(0.193,0.35,1,FALSE)`: Excel 0.9433773488001553, ours ...557. Suspected: Γ(a) for a < 1 through K1
  (T-2122) in the density prefactor, not T-2852's cut (T-3633 counts 0 visibly off here).
- probes-numeric/fit-audit-t3070-band-upper (11) and fit-audit-t3070-shape-min (11): `=GAMMA.INV(p,a,1)` at shapes
  1.5E+8 to 1E+10, 1-4 ulp of a quantile near a. E.g. band-upper S!A36 `GAMMA.INV(4.51E-13,9000000000,1)`: Excel
  8999322211.988121, ours ...117; shape-min S!A34 `GAMMA.INV(0.5,1.5E+8,1)`: Excel 149999999.66666666, ours
  149999999.6666667. Suspected: the huge-shape quantile path's last rounding (T-3070's band, statistics.md).
- probes-numeric/t3316-gamma-density-overflow-nonzero (4): GAMMA.DIST at scale 2.3E-308. S!A4
  `GAMMA.DIST(2.9E-308,5,2.3E-308,FALSE)`: Excel 1.297635969111423e+306, ours 1.2976359691114241e+306; S!A5
  `GAMMA.DIST(5.3E-308,10,2.3E-308,TRUE)`: Excel 0.00014612050406710144, ours ...018 (NumericClose, still within 15
  digits).
- None of the four cases has a math sidecar. Code: `crates/rustsheet-engine/src/functions/statistical/` (gamma
  distribution and inverse files). Every miss is below decision 0112's floor, so a form chosen on them needs the R4.4
  check or an owner ruling.

Cells (gap triage 2026-11-03m; 58 cells):
- probes-numeric/fit-audit-t2852-shape-min.case.json (32): S!A1:A2, A4, A7:A13, A15, A18:A19, A21:A24, A26:A27, A31:A33, A35:A36, A39:A45, A47
- probes-numeric/fit-audit-t3070-band-upper.case.json (11): S!A17:A19, A23, A25, A31, A35:A37, A42:A43
- probes-numeric/fit-audit-t3070-shape-min.case.json (11): S!A4, A6, A8, A10, A13:A14, A17, A34:A36, A39
- probes-numeric/t3316-gamma-density-overflow-nonzero.case.json (4): S!A2, A4:A6

## Acceptance criteria
- [ ] Each listed cell passes bit-exact, or the Outcome lists the ones still open with the kernel and the model tried.
- [ ] No GAMMA.DIST, GAMMA.INV, GAMMADIST, GAMMAINV, CHISQ.* cell of any suite regresses; the known-failure lines in
      eval/baseline/ only shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3939, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scoring the FITTED cuts themselves (T-3633) and the cells T-3396, T-3294 and T-3409 list.

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
