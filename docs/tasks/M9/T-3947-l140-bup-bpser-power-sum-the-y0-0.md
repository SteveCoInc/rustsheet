---
id: T-3947
title: "L140 bup+bpser power sum: the y0 = 0.5 points and the large-b0 cells it leaves off"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
BETA.DIST and the F tails on bratio's L140 bup+bpser branch agree with Excel to 15 significant digits at the points
T-3198's power sum leaves off: y0 = 0.5 with many bup terms, and the large-b0 cells of its fresh sweep.

## Context
T-3198 moved L140 bup+bpser's bup sum W from TOMS's recurrence to T-3568's power sum `1 + Σ R_i·EXP(fl(i·LN y0))`
(`bratio/bup.rs` `l140_bup_bpser`; statistics.md "Incomplete beta L140 bup+bpser at the F points"). That is the best of
the forms scored, but not Excel's exact arithmetic:
- At y0 = 0.5 with k >= 8 (TOMS's bup break), Excel is 1 to 3 ulp off the power sum and on TOMS's recurrence:
  `BETA.DIST(0.5,100,10,TRUE)` (probes-numeric m9-beta-dist-a100 A39, dropped from
  `bratio::tests::l140_whole_b0_takes_ln_beta_of_one`) and (10, 10) at 0.5 (skipped in
  `ported_tails_rise_with_x_where_excel_does`). At y0 = 0.5 with k = 3 or 6, Excel follows the power sum. Both
  carry `TODO(T-3947)`.
- probes-numeric t3198-l140-power-sweep, more than 4 ulp off Excel (the harness's NumericClose): Calc!A24, A45, A70, A85, A132, A155, A158, A173,
  A181, A204, A224, A241, A289, A356, A384, A394, A398. Most have b0 >= 20, so they sit deep in bup's sum (many terms).
  Main's recurrence left 16 of the case's cells visibly off.
- Candidates T-3198 ruled out are listed in statistics.md: x87-extended powers, libm `pow`, Excel's integer POWER,
  2^RN64(i·log2 y0), and exact powers at y0 = 0.5 only.

## Acceptance criteria
- [ ] The y0 = 0.5 cells above are bit-exact without losing the power sum's cells elsewhere (t3198-f-l140-args columns
      C and D: 674 of 816 bit-exact today; t3198-l140-power-sweep 259 of 400), or a measured reason why not.
- [ ] Every t3198-l140-power-sweep cell listed above is within 4 ulp of Excel, or named in the Outcome and handed
      to a task that owns it.
- [ ] Revisit BETA.INV's search on this branch (`BupSum::Recurrence`, `bratio::beta_search_tails`). Its only evidence is
      ironcalc-calc BETADIST_BETAINV.xlsx H32.
- [ ] Any cut or constant chosen by scoring meets decision 0188's R4 guards.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F argument form (T-3198's A19) and the F densities (`f_density.rs`).

## Deferred by decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) parks this task until Phase 1 exits. It is work on the incomplete-beta family whose cells are first wrong only at the 14th or 15th significant digit, or a probe or port that serves only that work. In those regions Excel itself is not correctly rounded (on a sample of 2,026 visibly-off cells, Excel was correctly rounded on 1%, with a median error of 48 to 330 ulp). Matching those digits means reproducing Excel's rounding noise. Decision 0267 accepts those cells. Reopen the task only if a cell it owns is first wrong at the 13th digit or earlier.
