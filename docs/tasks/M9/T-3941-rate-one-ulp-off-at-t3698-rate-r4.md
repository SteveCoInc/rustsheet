---
id: T-3941
title: "RATE one ulp off at t3698-rate-r4-fresh A233 (nper 795, guess 0.124; Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
`RATE(795,-104389064.210271,53339676251.2274,0,0,0.124098481012326)` returns Excel's 0.0012068938114889573, with no
RATE cell regressing.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (RATE) measured on t3850-rate-same-state, 0 of 240 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; per-cell `owned` run over every
  failing cell against every open task, deferred ones included, and every unmerged branch's task files).
  probes-numeric/t3698-rate-r4-fresh is T-3698's R4.2 fresh sweep (300 cells, ungated). T-3876 owns its A201 (the
  guess returned when every residual is rounding noise) and T-3906 (done) scored it for flat secants; Calc!A233 has no
  owner.
- Calc!A233: Excel 0.0012068938114889573, ours 0.0012068938114889575 (NumericUlp, 1 ulp, within 15 digits). nper
  795 and a guess of 0.124, far from the root, so the iteration takes many steps; the last digit depends on Excel's
  stopping rule and update order (financial.md, the RATE entries of T-3698 and T-3906).
- Code: `crates/rustsheet-engine/src/functions/financial/` (RATE's secant/Newton loop). A single 1-ulp cell is below
  decision 0112's floor: do not add a branch for it. Fix it only through a general change to the iteration that the
  RATE probes support, or record it as a negative.

Cells (gap triage 2026-11-03m; 1 cell):
- probes-numeric/t3698-rate-r4-fresh.case.json (1): Calc!A233

## Acceptance criteria
- [ ] Calc!A233 passes bit-exact through a change the RATE probes support, or the Outcome records the iteration
      traces (ours and the model's) and why it is left.
- [ ] No RATE cell of any suite regresses; the known-failure lines in eval/baseline/ only shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3941, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A201 (T-3876).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
