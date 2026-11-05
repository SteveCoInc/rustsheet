---
id: T-4214
title: "BETA.DIST at b = 40 and a first shape of 2E9 to 1E10: Excel is 5E-10 to 4.6E-8 off the math, ours on it (28 t4080 cells, 26 at the 8th to 10th digit, decision 0267 Tier A)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
`BETA.DIST(x,a,40,TRUE)` at a first shape of 2E9 to 1E10 returns Excel's value on the 28 listed cells. Excel is the
inaccurate side there: it is 5E-10 to 4.6E-8 off the 60-digit math, and the engine is within 1E-14. The task finds the
arithmetic Excel uses on this path, most likely TOMS 708's bfrac branch, and reproduces it. If two attempts fail, the
task blocks as time-boxed (decision 0056).

## Context
- probes-numeric/t4080-beta-num-band-edge (T-4080, excel-graph 2026-11-04) probed `BETA.DIST` at 14 first shapes
  (2E9 to 1E10) by b = 30 to 40 by 1/2, at four tails. At b = 40 every cell is a number, outside the `#NUM!` band that
  T-4044 owns. Columns A and B (x = 1 - q/a, I about 1E-6 and 0.05) are off. Columns C and D are not.
- These 28 cells were owned only by T-3209, which is `defer: phase-1b` (its subject is Excel's bgrat digits at
  14th-15th digit size). So `make unowned-off ALL=1` lists 26 of them as `BETA.DIST (deferred)`. A63 and A147 are left
  out: Excel is within 1E-9 of the math there, so they count as Phase 1b evidence. Under decision 0267 item 2, these
  family cells are first wrong at the 8th to 10th significant digit, which makes 26 of them Tier A. Decision 0267 says a
  Tier A cell needs a live owner, and this task takes them from T-3209.
- Gap triage, 2026-11-04: Excel, the engine and 60-digit mpmath `betainc(a, 40, 0, x)` on the double x.
  Excel's relative error is 4.5E-10 (A63) to 4.6E-8 (B294). Ours is 7E-18 to 9E-15. For example:
  - A105 `BETA.DIST(0.999999974153246,3000000000,40,TRUE)`: Excel 9.999988703240594E-7, ours 9.999988650466833E-7,
    math 9.9999886504667962E-7 (Excel +5.3E-9).
  - B294 `BETA.DIST(0.999999994906026,10000000000,40,TRUE)`: Excel 0.049999949775626405, ours 0.049999952069376706,
    math 0.049999952069376659 (Excel -4.6E-8).
  - B231 (a 6E9): Excel 1.7E-8 off. B42 (a 2.25E9): Excel 1.3E-8 off. The size of the error does not grow steadily with a.
- Excel stays the target: its error is below decision 0188 R3's G = 1E-4.
- Hypothesis: the path. TOMS 708 bratio, as `bratio::dispatch` in crates/rustsheet-engine/src/functions/statistical/bratio/mod.rs
  writes it:
  - λ = (a + b)·y - b > 0 on columns A and B, so there is no swap: a0 = a, b0 = 40.
  - b0 < 40 is false, so the point leaves L140 (bup + bgrat), which is why b = 40 is outside the `#NUM!` band
    (statistics.md "The edge (T-4080)").
  - a0 > b0 and b0 <= 100, so the point goes to `Branch::Bfrac`.

  bfrac's prefactor brcomp takes its large-parameter form at a0 >= 8. There, a0·rlog1(-λ/a0) multiplies the rounding of
  its inputs by about a0, which is 1E9 to 1E10. That gives an error of about 1E-7 to 1E-6 relative, the same scale as
  Excel's error. First check: evaluate the engine's own `bfrac` port (bratio/bfrac.rs, which has Excel-matched tests
  `bfrac_matches_excel` and `bfrac_lentz_matches_excel`) at these 28 points, with brcomp in plain doubles. Compare the
  results to Excel's bits before writing anything new. Then find which kernel BETA.DIST takes there today:
  `large_shape_beta` (T-4119) covers only b < 40.
- The same branch at the F functions: probes-numeric/t4030-f-dist-rt-df1-boundary A39, A40, A61 and A62
  (`F.DIST.RT(2,80|100,1E10|5E9)`, b0 = 40 and 50). There Excel is 9E-10 to 7E-9 off the math and ours 1E-8 to 1.4E-7.
  Those cells stay T-4044's. Use them as a cross-check: a model of Excel's bfrac at a huge a0 should move them toward
  Excel too. Report the effect in both tasks.
- docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape" (the T-4030 and T-4080 paragraphs).
  Record the finding there.
- Probe freeze (decision 0267 item 5): this task has a "Decision 0267 cells" section, so it may add a probes-numeric
  case for the R4 fresh sweep of these cells' region.

## Acceptance criteria
- [ ] Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit. Otherwise two
      attempts are recorded and the task blocks as time-boxed (decision 0056), and decision 0267 accepts what is left.
- [ ] statistics.md states Excel's arithmetic at b0 = 40 to 100 and a huge a0 (or what was ruled out), citing these
      cells, and the engine change carries a comment that cites the same evidence.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7). A port of a published TOMS 708 routine as written is
      kind (b). Any rounding or operation-order model chosen by scoring is kind (c) or (a):
  - [ ] Hold-out: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        candidate and the correctly rounded value, from the fit lab's split or a stated manual split of about 30% by
        cell hash, with the margin fixed before scoring. The chosen candidate beats correctly rounded on hold-out.
  - [ ] Fresh sweep: after the model is frozen, a new log-uniform probes-numeric sweep over a0 1E7 to 1E10,
        b0 40 to 100 and tails 1E-8 to 0.5 on the λ > 0 side, oracle-filled. Its result is recorded in the Outcome.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope, with every point outside listed, and no BETA.DIST,
        F.DIST or F.DIST.RT Gate A point moved out of it (`rustsheet-eval accuracy`).
  - [ ] `FITTED(T-4214, n_cells, range)` on each fitted piece (kind (a) only). Outside its range, the code uses accurate
        math.
  - [ ] Reproducibility floor: no step targets differences below 4 ulp web against desktop, or bits that vary
        between fills (decision 0112).
  - [ ] No keep-matched branch: the branch is selected by TOMS's dispatch (b0 >= 40, a0 > b0, b0 <= 100, or whatever
        the evidence pins), never by the set of cells that already match.
- [ ] No cell of columns C and D, of b < 40, or of any gated suite moves off. Family regressions only under decision
      0267 item 6. `make check-fast` is green locally, CI is green, and reviewers approve (excel-fidelity included).

## Out of scope
- The `#NUM!` band at b < 40 and the F cells of t4030 (T-4044).
- Excel's bgrat digits at b < 40 (T-3209, deferred), and the three t4080 cells at b < 40 that are only 13th-digit
  residue (A38, A223, A283).
- Our F accuracy at a huge df2 (T-3754).

## Decision 0267 cells
Gap triage (origin/main b450050f8) gives this task the 28 b = 40 cells of t4080 columns A and B. Before, the only owner
was T-3209 (deferred). 26 are Tier A: first wrong at the 8th to 10th significant digit under decision 0267's 13-digit
family bar. A63 and A147 are first wrong at the 10th digit, but Excel is within 1E-9 of the math there, so they are
Phase 1b evidence (decision 0188 R1(c)). Decision 0056's time-box applies.

- probes-numeric/t4080-beta-num-band-edge.case.json (28): Calc!A21, B21, A42, B42, A63, B63, A84, B84, A105, B105, A126, B126, A147, B147, A168, B168, A189, B189, A210, B210, A231, B231, A252, B252, A273, B273, A294, B294

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit (decisions 0083, 0267).
