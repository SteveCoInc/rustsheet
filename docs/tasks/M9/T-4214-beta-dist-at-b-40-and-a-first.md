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
- Our F accuracy at a huge df2: T-3754 (done) and, at b0 >= 40, T-4435 (our F route is 4.7E-8 off the math at
  `FDIST(1.5,80,1E10)`, where our BETA.DIST at the same shapes is on it).

## Decision 0267 cells
Gap triage (origin/main b450050f8) gives this task the 28 b = 40 cells of t4080 columns A and B. Before, the only owner
was T-3209 (deferred). 26 are Tier A: first wrong at the 8th to 10th significant digit under decision 0267's 13-digit
family bar. A63 and A147 are first wrong at the 10th digit, but Excel is within 1E-9 of the math there, so they are
Phase 1b evidence (decision 0188 R1(c)). Decision 0056's time-box applies.

- probes-numeric/t4080-beta-num-band-edge.case.json (28): Calc!A21, B21, A42, B42, A63, B63, A84, B84, A105, B105, A126, B126, A147, B147, A168, B168, A189, B189, A210, B210, A231, B231, A252, B252, A273, B273, A294, B294

Gap triage 2026-11-06 (origin/main 39f04d556) adds `FDIST(1.5,80,1E10)`, which is 1 - I(40, 5E9) on this path
(a0 = 5E9, b0 = 40). Before, it was listed only by deferred T-3209, and T-4254 named its case in passing (T-4436).
- Excel 0.0025481923984377957 is 1.0E-8 off the math 0.0025481924241211687 (60-digit mpmath at the exact ratio).
- Ours 0.002548192305228901 is 4.7E-8 off, through the F route that T-4435 makes accurate.
- So the cell needs both: T-4435 brings ours to the math, and this task's model brings it to Excel.
- probes-numeric/t4045-f-dist-rt-zero-cut.case.json (1): Calc!E3

## T-4434 cells
T-4434's sweep (docs/research/t4434, `score.py` `owner`) gives this task the 342 cells where ours is not within 15
digits of Excel, the correctly rounded math is not either (so Excel is the side off the math), and bratio's b0 is 40
or more (bfrac). They are region 2's F tails above the mean (df1 80 to 400, df2 1E5 to 1E10), F.DIST at the same
points, and BETA.DIST above the mean at a of 40 and more, where bfrac runs swapped with the huge shape as a0. Excel
is more than 1E-13 off the math on 261 of the 342, from 1E-14 to 1E-13 on 42 and within 1E-14 on 39 (its bfrac
digits, which the correctly rounded math does not match to 15 digits); up to 8.5E-9 (B), 1.3E-9 (C) and 3.5E-9 (D),
growing with a0 (statistics.md "Incomplete beta at
a huge first shape", T-4434 item). T-4435 moves ours toward the math first. These are evidence cells, not Decision
0267 cells: decision 0267's tiers and time-box do not extend to them.
- probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json (342): Calc!B1:B11, B13, B15:B16, B18, B22:B25, B27, B29:B31, B33, B36:B39, B41, B44:B51, B53:B58, B60:B61, B63:B64, B66:B71, B73:B74, B76:B77, B79, B82:B84, B86:B88, B90:B93, B96:B97, B99, B101:B106, B108:B112, B114:B116, B118:B119, B122:B123, B125:B127, B130:B131, B133:B139, B141:B142, B144, B146:B151, B153, B155:B160, B163:B189, B191:B192, B194:B197, B199:B209, B211:B212, B214, B217, B219:B229, B231:B237, B239:B241, B243:B244, B248:B251, B253:B255, B257, B260:B263, B267:B272, B274:B275, B277:B278, B281, B283, B285:B291, B293:B294, B296:B300, C2, C9:C10, C15, C18, C20, C23, C30:C31, C33, C36:C38, C41, C44:C46, C48:C49, C56, C61, C67:C68, C74:C75, C77, C79:C80, C83:C84, C88:C89, C91, C93, C102, C105:C106, C108, C112, C115, C120, C125, C129, C131, C134:C136, C140:C141, C149:C150, D1:D2, D5, D7, D9, D12, D14:D16, D22, D24:D26, D28, D34:D36, D39:D43, D46, D55:D56, D62, D64, D69:D70, D74:D75, D78:D79, D81, D87, D90:D91, D94, D98:D101, D103:D104, D106:D107, D109:D111, D113, D119:D123, D130, D133, D135, D137:D139, D142, D147:D148, D150

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit (decisions 0083, 0267).
