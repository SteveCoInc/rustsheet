---
id: T-3936
title: "BETA.DIST cdf on the bfrac Lentz region once the density matches: the 1,157 unowned t3571-r5-bfrac-fresh cdf cells (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3935]
campaign: visible
defer: phase-1b
---

## Goal
`BETA.DIST(x,a,b,TRUE)` on the bfrac Lentz region (a, b in [40, 5000], z in [-8, -0.2]) is within half a unit of the
15th significant digit of Excel's value on the 1,157 cdf cells of T-3571's fresh sweep listed below, once T-3935 gives
Excel's density there.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (incomplete beta, bfrac) measured on t3953-f-tail-bfrac-tiny, 0 of 4800 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; per-cell `owned` run over every
  failing cell against every open task and every unmerged branch's task files). T-3812 (done) ported Excel's Lentz
  loop and used this case as its fresh sweep; no open task owns its cdf column except T-3680's four cells with no
  math value (A239, A441, A902, A978, left out here).
- probes-numeric/t3571-r5-bfrac-fresh (ungated), column A `=BETA.DIST(x,a,b,TRUE)`: failing 1,157 of 1,200 (705
  visibly off; all NumericClose or NumericUlp). Against the 80-digit math sidecar Excel is a median 38 ulp (p90 201)
  from the math and the engine 32 (p90 166), both within 1e-9: Phase 1b evidence under decision 0188 R1(c), stage
  `visible`, deferred like T-3244 and T-3474.
- Root cause (suspected, from T-3812's Outcome): the cdf is fl(brc·f) with brc built from the density, and the
  engine's density equals Excel's on only 32 of the 1,200 points. With Excel's density, T-3571's `model.py`
  reproduces 694 of the 1,200 fresh cdf cells bit-exact, against 39 for the engine now. So most of this column
  moves with T-3935; this task owns what is left: the brc prefactor and the Lentz loop's last digits.
- Examples, `make eval-show CASE=probes-numeric/t3571-r5-bfrac-fresh.case.json`:
  - Calc!A1 `=BETA.DIST(0.732672756053141,2500.0,700.0,TRUE)`: Excel 1.0819773476209992e-10, ours
    1.0819773476210277e-10.
  - Calc!A10 `=BETA.DIST(0.844587793975625,726.67,110.92,TRUE)`: Excel 0.028728219997512752, ours
    0.028728219997512735.
  - Calc!A100 `=BETA.DIST(0.281085972859859,125.5,173.982,TRUE)`: Excel 1.7242594930046796e-07, ours
    1.7242594930046735e-07.
- Code: `crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs` (`lentz_fraction`, `lentz_region`);
  research docs/research/t2464/r5-bfrac. F.DIST and BINOM.DIST on the same region are T-3913's.
- Fitting: start by re-scoring after T-3935 lands. This case was the fresh sweep of T-3571's frozen model, so any new
  fit needs a fresh sweep of its own.

Cells (gap triage 2026-11-03m; 1,157 cells, all BETA.DIST(.,TRUE)):
- probes-numeric/t3571-r5-bfrac-fresh.case.json (1157): Calc!A1:A8, A10, A12:A87, A89:A188, A190:A202, A204:A238, A240:A286, A288:A320, A322:A326, A328:A353, A355:A375, A377:A391, A393:A401, A403, A405:A428, A430:A440, A442:A469, A471:A475, A477:A517, A520:A530, A532:A548, A550:A570, A572:A578, A580:A674, A676:A699, A701:A778, A780, A782:A783, A785:A901, A903:A911, A913:A945, A947:A952, A954:A977, A979:A982, A984:A1012, A1014:A1032, A1034:A1037, A1039:A1054, A1056:A1141, A1143:A1157, A1159:A1169, A1171:A1194, A1196:A1200

## Acceptance criteria
- [ ] The listed cells are within 15 digits of Excel's value, or each one left over is named in the Outcome with its
      reason and an owner.
- [ ] No new visibly-off cell in any suite; the known-failure lines in eval/baseline/ only shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3936, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] statistics.md's "Incomplete beta bfrac is TOMS's fraction in a Lentz loop" entry records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The density (T-3935); F.DIST and BINOM.DIST on the Lentz region (T-3913); the four no-math cells (T-3680).

## Visible-parity stage
Target (decision 0114): every cell above within half a unit of the 15th significant digit of Excel's value. Bit-level
residue goes to a follow-up with `defer: phase-1b`.
