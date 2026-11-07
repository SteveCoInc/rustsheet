---
id: T-2006
title: "Least-squares kernel for LINEST, LOGEST, TREND and GROWTH: fit Excel's evaluation order (about 300 last-digit cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The least-squares solve behind LINEST, LOGEST, TREND and GROWTH gives Excel's bits: one fitted kernel replaces the
normal-equations solve, so the four functions' last-digit failures go together. For example, `TREND(A2:A6)` should give
0.18400000000000014, as Excel does, not the exact 0.184.

## Context
Gap triage 2026-10-19. The four functions share `linear_regression` (functions/statistical/linest.rs). It forms D'D
and D'y and solves them with Gauss-Jordan (`solve_system`). Excel's values carry rounding error that this does not
reproduce, even on exactly representable fits. statistics.md "LINEST, LOGEST, GROWTH: probe findings":
`LOGEST({1,2,4,8},{1,2,3,4})` is 1.9999999999999998 and 0.5000000000000001, `GROWTH(5,5,5)` is 4.999999999999999.
Microsoft's LINEST help describes a QR decomposition since Excel 2003. Treat that as a lead, not evidence: the fit
decides which decomposition it is, if any (Householder or Gram-Schmidt, column order, pivoting, centring).

Numeric cells (both values numbers, verdict ulp, close or mismatch), 303 in total:
- probes/linest-modes 81 (38 ulp, 32 close, 11 mismatch), e.g. collinear `LINEST(B2:B9,A2:E9,TRUE,TRUE)`.
- probes/logest-modes 84 (20 ulp, 63 close, 1 mismatch).
- probes/growth-modes 57 (28 ulp, 29 close).
- probes/regression-trend 15 ulp, e.g. O2 `INDEX(TREND(A1:A6,B1:B6),1,1)` expected 2.571428571428571, ours
  2.5714285714285694, and O3 expected 7.2, ours 7.200000000000003.
- ironcalc-calc/STATISTICAL/TREND_GROWTH.xlsx 26 and LINEST_LOGEST.xlsx 24 (desktop cache, Mac 16.x), e.g.
  TREND_GROWTH Sheet1!F2 `TREND(A2:A6)` expected 0.18400000000000014, ours 0.184, and LINEST_LOGEST Sheet1!E3
  `LINEST(A2:A6)` intercept expected -0.031000000000000028, ours -0.03100000000000025.
Each function's own task keeps its shape, flag, error and stats-layout cells: T-1741 (LINEST), T-1742 (LOGEST),
T-1743 (GROWTH), T-1726 (TREND). The numerics were in those tasks too, so four sessions would have fitted the same
kernel. Decision 0056 says to fit the kernel first: one lab fit over all of them. This needs numeric fitting against
several hundred cells, hence opus.

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: LINEST and GROWTH last digits (the shared kernel). Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/285d87cea1aaa73c Calc!A183 `=LINEST(Data!E11:E12)`: Excel -1e-15, ours -9.999999999999997e-16
Cells:
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A183
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A178
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A341

Also from T-1741 (visibly off, the solve's noise digits): probes/linest-modes K106:P110 (collinear `LINEST(B2:B9,
A2:E9,TRUE,TRUE)`: Excel 1.4e-16-sized coefficients and se, F 3.1e34 at K109 where ours is exactly 0 and `#NUM!`),
K190:L194 and K197:L201 (y in x: F 1.36e32 at K193, K200, ours `#NUM!`), and K134:L138 (two points: Excel exactly 0.5,
-1.5 and ss_resid 0; ours 0.49999999999999867 and 3.6e-30). Excel's exact or collinear fits carry its solve's residual
noise, so F is a number there; ours is exact, so F is `#NUM!`.
The singular test is an absolute pivot under 1e-12 (`solve_system`, `invert_matrix`) while `independent_columns`
drops on a relative 1e-10, so a badly scaled x (values near 1e-7) can fail the solve with nothing to drop and give
`#NUM!` where Excel would fit: the new kernel should test singularity relative to scale.

Also from T-1742 (visibly off, the solve): probes/logest-modes K85 `LOGEST(D2:D9,A2:C9,TRUE,TRUE)` (Excel
0.982454941667086, ours 0.9824549416670796, past the 15th digit), the exact-fit stats K169:L173 and K197:L201
(`LOGEST({1,2,4,8},{1,2,3,4},TRUE,TRUE)`, `LOGEST({0.5,0.25,0.125},{1,2,3},TRUE,TRUE)`: Excel's se and ss_resid are
exactly 0 and F `#NUM!`; ours carry 2E-16 and 1E-31 of noise and F 1E+31), and gen-m9 589f42e67f4b4b8d Calc!A193
`LOGEST(Data!A10:B11)` (Excel 2.220601291150046, ours 2.2206012911500514). Unlike LINEST's exact fits, LOGEST's ln(y)
is exactly linear here only to rounding, so a kernel that gets these to exactly 0 is a strong lead.

Also from T-1743 (visibly off, the solve): probes/growth-modes K145:K152 (`GROWTH(D2:D9,A2:C9,A2:C9)`, e.g. K147 Excel
9.034855848946608, ours 9.034855848947016), K157 and K169 (a multi-column x with `{1,2,3}` as new_x), and K3, K15, K27,
K123, K219 (Excel 9.215274223288255, ours 9.21527422328826); gen-m9 8fec7b207f4dd753 Calc!A75 `GROWTH(Data!D10:E10)`
(Excel 0.19999999999999996, ours 0.2000000000000001, a last-digit line).

## Acceptance criteria
- [ ] A fit-lab kernel (tools/src/rustsheet_tools/fitlab/kernels/, a new `regression` fit) scores named candidates
      on the cells above. Candidates: normal equations (today's), Householder QR, modified and classical Gram-Schmidt,
      centred data with and without a separate intercept, and LOGEST/GROWTH taking ln(y) before or inside the solve.
      Each candidate's bit-exact and within-4-ulp counts go into statistics.md, rejected ones included.
- [ ] The best candidate is implemented once in linest.rs and used by all four functions. No baseline gains a line.
      The F lines for the ulp/close cells of linest-modes, logest-modes, growth-modes, regression-trend, TREND_GROWTH
      and LINEST_LOGEST shrink, and the PR lists how many leave each case. If two attempts do not raise the bit-exact
      count, the task is set `blocked` with "Blocked because: time-boxed (decision 0056)".
- [ ] Stats rows (se, r2, F, ss) use the same decomposition, and the PR states which of them now match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Result shapes, `#N/A` fill, flags, omitted arguments and error choice (T-1741, T-1742, T-1743, T-1726).
- FORECAST/FORECAST.LINEAR, SLOPE, INTERCEPT, STEYX and the CORREL family. They are closed forms in forecast.rs and
  correl.rs, not this solve (T-1725, T-1723, T-1707).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Cells added by gap triage (2026-10-31m)
`make eval` on origin/main 900fbc1bf (excel-graph). An ownership check over every failing Excel cell
(`scripts/exit_owners.py`'s `owned` rule over the open task files of origin/main and of every `origin/agent/*` branch
pushed since 2026-10-29) found these cells named by no open task. All are within 15 significant digits of Excel (Phase
1b), so they join this deferred owner of their kernel. LINEST, LOGEST, TREND and GROWTH with known_y only (range and
array forms) on the stat argument-modes probe, 1 to 6 ulp: `INDEX(TREND(A1:A8),1,1)` Excel 1.4999999999999996, ours
1.5000000000000009.
- probes/stat-argument-modes.case.json (8): Sheet1!H318, H319, H321, H322, H324, H325, H327, H328

## Cells added by gap triage (2026-11-05)
`make eval` on origin/main 4e3993c9e: the same ownership check over every failing Excel cell found one more LINEST
cell no open task names, 4 ulp and within 15 digits (Phase 1b). It sits in an AMORDEGRC state probe but calls
LINEST alone: `SUM(LINEST({1,2,3},{1,2,4}))` is Excel 1.1428571428571428, ours 1.1428571428571437.
- probes/amordegrc-fraction-state-cc.case.json (1): Sheet1!A2

## Points added by T-4096 (Gate A tables, 2026-11-06)
The Gate A tables accuracy/{LINEST,LOGEST,TREND,GROWTH}/s1 (excel-graph) found 21 points where the normal-equations
solve is visibly off on an ill-conditioned design (large offsets with a small relative spread, mixed magnitudes),
from 2E-9 to 20 relative off the exact least-squares value, while Excel is within 2E-8 of it on 19 of them (LOGEST
A82's and A98's near-exact-fit F and ss_resid are 8E-7 and 3E-6 off in Excel too). TREND A193 is the badly scaled
`#NUM!` described above (data from 1E-45 to 1E+49; Excel and the math -1.3025416638036628E+49). So a solve with
Excel's conditioning (QR rather than the normal equations, and a singularity test relative to scale) brings the other
19 inside the envelope whichever Excel bits it reaches (LOGEST A82 and A98 need Excel's own digits to 15 places); per the Deferred section above these visibly-off points move the task from
`defer: phase-1b` to `campaign: accuracy`. They are lines of eval/accuracy/known-failures.tsv owned by this task:
- accuracy/LINEST/s1: Calc!A85, A87, A93, A98, A100, A102, A103, A104, A110, A129, A187
- accuracy/LOGEST/s1: Calc!A81, A82, A86, A98, A103, A110
- accuracy/TREND/s1: Calc!A94, A98, A193
- accuracy/GROWTH/s1: Calc!A97

## Landing notes
Opus run 2026-11-06 (13:38–15:28 UTC). No zero-regression kernel was found within the session, so there
is no PR. The branch holds a scoring scaffold in linest.rs (`KERNEL` const, `householder`, `fit_kernel`), not a
change to merge. These notes are data for whoever resumes the branch, not instructions.

How the variants were scored: a release build per variant, a full `rustsheet-eval run --out <dir>`, then the
failing cells whose formula names LINEST, LOGEST, TREND or GROWTH, diffed against origin/main e28570d3. That run has
147 such failing cells (92 ulp, 45 close, 6 error_mismatch, 3 numeric_mismatch, 1 type_mismatch). Gate A
(`rustsheet-eval accuracy`) has not been run on any variant yet.

| KERNEL | const=TRUE | const=FALSE | fixed | newly failing |
|---|---|---|---|---|
| 0 | normal equations, Gauss-Jordan (main) | same | — | — |
| 1 | centre x and y, normal equations on the centred cross products, b0 = ym - b·xm | Householder (later: normal equations) | 60 | 3 |
| 2 | centre x and y, Householder on the centred x only | normal equations | 60 | 4 |
| 5 | centre x only, Householder on [1, x - xm] (intercept column first), R's diagonal read from the reflected column, b0 = c0 - b·xm | Householder on raw x, R's diagonal -sign·norm | 83 | 19 |
| 6 | as 5 | Householder, diagonal from the reflected column | 82 | 39 |
| 7 | as 5 with the -sign·norm diagonal | as 5 | 74 | 19 |
| 8 | as 5, but when no x column is kept the intercept is mean(y) | as 5 | 83 | 15 (147 failing cells down to 79) |
| **9 (the scaffold's setting, best so far)** | centre x **and** y; Householder on [1, x - xm] (ones column first) against y - ym, with the -sign·norm diagonal; b0 = ym - Σ b_j·xm_j; when no x column is kept, b0 = mean(y) | Householder on raw x, -sign·norm diagonal | **126** | **3** (147 failing cells down to 24) |

Householder here is the textbook v = a - alpha·e1, H = I - 2vv'/v'v, alpha = -sign(a_jj)·‖a_j‖. The LINPACK
(`dqrdc`) and LAPACK (`dlarfg`, tau) reflectors were also tried, by hand in Python on 10 cells, and were worse.

Python hand checks on Excel cells (all Excel values are oracle cells):
- KERNEL 5's const=TRUE form matches all 5 cells tried:
  - linest-modes `LINEST(B2:B9,A2:A9)` slope 1.9976190476190474;
  - `LINEST(A2:A9,A2:A9)` 1.0000000000000002 and `LINEST(A2:A9,E2:E9)` 0.5000000000000001. Excel's noise on these
    exact fits rules out every form that centres y, all of which give exactly 1 and 0.5;
  - regression-trend O2 2.571428571428571;
  - `SUM(LINEST({1,2,3},{1,2,4}))` 1.1428571428571428.
- Its const=FALSE form matches all 5 cells tried:
  - t4222 TREND 7.200000000000001 and slope 1.2000000000000002;
  - linest-modes K43 2.0039215686274505;
  - regression-trend N8 and O14.
- The plain normal equations get 0 of the 5 const=TRUE cells and 3 of the 5 const=FALSE cells.

KERNEL 5's 19 new failures, 15 under KERNEL 8, which fixes the four constant-x cells (`TREND(A1:A6,G1:G6)` O20, O21, N20, N21). These cells pass on main, so they are regressions that the eval gate would refuse:
- regression-trend: 15 cells where Excel is exact and KERNEL 5 is 1 to 6 ulp off. Excel's values: SUM(TREND(A1:A6)) 27,
  `TREND(A1:A6,G1:G6)` 4.5 (G constant, so x is dropped), SUM(TREND({2,4,5},{1,2,3})) 11, O34 3.6666666665000003,
  O44 10000000007.466667 (I is 1E10 plus small offsets). So Excel's intercept, or its prediction, carries no QR noise
  where the fitted mean is exact. KERNEL 5's c0 for A1:A6 is 4.499999999999999, but Excel's is 4.5.
- linest-modes K183 (a stats cell; the stats rows still use the old (D'D)^-1 and are not ported yet), K85
  (collinear: Excel gives 0, KERNEL 5 gives -1.6E-16), and gen-m9 8fec7b207f4dd753 A94.

Next step I would take: keep KERNEL 5's slopes, but take the intercept and the TREND/GROWTH prediction from the exact
means: b0 = ym - Σ b_j·xm_j, and predictions ym + Σ b_j·(x_j - xm_j) or b0 + Σ b_j·x_j. Score those forms first on the
15 regression-trend cells above and on O2 in Python, then run Gate A. Then port the stats rows to R (se from R^-1, ss_resid from the
residuals of the same solve), turn the scaffold into the fitlab `regression` kernel the acceptance criteria ask
for, and record the candidate table in statistics.md.

KERNEL 9: the best candidate so far. I found it with a Python grid over 133 single-predictor oracle cells. The cells
come from linest-modes, logest-modes, growth-modes, regression-trend, stat-argument-modes, t3185-b and
amordegrc-fraction-state-cc, every cell whose formula is a bare, INDEX- or SUM-wrapped call with literal or range
arguments. The grid crossed these options:
- reflector: textbook, LINPACK, LAPACK, or the textbook with R's diagonal read from the reflected column;
- centring: x, y, both or neither;
- the ones column: first, last or none;
- the intercept: from the QR solve or from the means;
- the prediction: b0 + b·x, ym + b·(x - xm) or c0 + b·(x - xm);
- the mean: a plain sum/n, or the corrected, fsum or pairwise means;
- the norm: a plain sqrt of the sum of squares, or dnrm2's scaled form.

The scores, as bit-exact cells out of 133:
- KERNEL 9's form: 113 bit-exact and 6 more within 4 ulp. The 14 it does not score are the one-point and
  constant-x cells that the drop path sends to b0 = mean(y), and those pass in the full eval. With const=FALSE it
  gets 23 of 23.
- Every other form: 109 or fewer. The normal equations: 37.
- The corrected, fsum and pairwise means for y: 102. Excel takes y's mean as a plain sum/n.
- dnrm2's scaled norm: 96. The plain sqrt of the sum of squares stays.

KERNEL 9's 3 newly failing cells, all 1 ulp and all one shape: regression-trend O34, O35 and N35, `TREND(A1:B3,H1:I3)`
with x = {6, 1E10+1, 5, 1E10+2, 4, 1E10+4}. Excel gives 3.6666666665000003; KERNEL 9 gives 3.6666666665. None of the
mean or norm variants fixes them. The 6 other single-predictor misses are GROWTH({1,2,4,8},{1,2,3,4},{5,6}) K289,
K301 and K325 (Excel 15.999999999999996, KERNEL 9 3 ulp below) and these three.

Gate A on KERNEL 9 (`rustsheet-eval accuracy`, 1.4 s):
- 16 of the 21 T-2006 points are now inside the envelope; they are stale known-failure lines to remove. Still known:
  LINEST A102 and A103, LOGEST A82, A86 and A98.
- One new failing point, which the gate refuses: LINEST/s1 A184,
  `INDEX(LINEST({-8.1E-35,-2.3E+19,2.1E+7,-2.0E-08,1.1E-10},{1.2E-10,8.8E+33,2.1E-17,1.4E+13,1.4E-44},TRUE,FALSE),1,2)`
  (the intercept, x from 1E-44 to 1E+33).

Remaining work before a PR:
1. Fix or explain the 3 regression-trend cells and LINEST A184. A KERNEL 9 that keeps them failing cannot merge: the
   eval and accuracy gates refuse new failures.
2. Port the stats rows (se, r2, F, ss) to the same decomposition. They still use (D'D)^-1 of the uncentred design.
   linest-modes K183 is one of these.
3. Turn the grid (the session's scratch Python is not committed) into the fitlab `regression` kernel. Record the
   candidate table and the "Excel centres x and y" finding in statistics.md, then do the R4 hold-out and the fresh
   sweep (this is a structural-model choice, so it owes them but needs no `FITTED` tag).
4. Replace the `KERNEL` scaffold with the one chosen form. Today it is dead-code variants, and clippy has not been run
   on it.

LINEST A184 in detail: the intercept is pure cancellation. ym is -4.65E18 and b·xm is about the same, so one ulp of
b·xm is 1024 to 2048. The values:
- Excel 5206016, math 5205576.09 (Excel is 8.4E-5 off the math);
- KERNEL 9 5208064 (4.8E-4 off);
- the reflected-column diagonal 5203968, LINPACK 5208064, LAPACK 5209088;
- ym + c0 - b·xm (c0 is the ones-column coefficient) 5207040.

The table has no slope point for this formula, so Excel's b is not known to the last bit. A table point holding this
call's slope (index 1,1) would decide between the reflector forms. I would probe it before fitting further.

Resumed 2026-11-07 00:20 UTC by an opus run (work-loop step (2): branch idle 9 h).

## Landing notes (2026-11-07, second opus run)
Resumed 00:20 UTC, stopped 00:50 UTC at the session's start-new-work limit. No PR: the gates still refuse the branch.
Measured on this branch's own baseline (cut before main's newer probes; merge `origin/main` before scoring next time),
`rustsheet-eval check`, all suites:
- KERNEL 9 as the first run left it: fixed 218 (probes 183, ironcalc-calc 31, gen-m9 4), REGRESSIONS 28 (probes 26,
  ironcalc-calc 2). 18 of the 26 were probes/t4222-regression-flag-ranges stats rows, which the first run's
  147-cell LINEST-family diff did not include (the probe postdates it): KERNEL 9's coefficients mixed with the old
  normal-equations residuals.
- **Committed now: stats rows from the same QR.** ss_res is the squared tail of Q'(y - ym) (the reflected right-hand side
  below row k), ss_reg = ss_tot - ss_res, r2 = ss_reg / ss_tot, se_y = sqrt(ss_res/df), F = ss_reg/(ss_res/df).
  Python on t4222 Calc!A1 `LINEST({2,4,5,4,5},{1..5},TRUE,TRUE)`: all five of Excel's values bit-exact (r2 0.6, se_y
  0.894427190999916, F 4.499999999999999, ss_reg 3.5999999999999996, ss_res 2.4000000000000004). Rejected: ss_res from
  the residuals y - (b0 + b·x), y - (ym + b·(x - xm)) or (y - ym) - b·(x - xm) (each 1 ulp off on ss_res), and r2 = 1 -
  ss_res/ss_tot (0.6000000000000001 or 0.5999999999999999). ss_reg = ‖head of Q'(y - ym)‖² ties with ss_tot - ss_res here.
  Result: fixed 308 (probes 266, ironcalc-calc 38, gen-m9 4), REGRESSIONS 9.
- Remaining 9 regressions:
  - linest-modes L85, M85 (`LINEST(B2:B9,A2:C9,TRUE,TRUE)`, y is one of the x columns: Excel exactly 1 and 0; ours
    0.9999999999999999 and 2.7E-16) and L106 (`LINEST(B2:B9,A2:E9,TRUE,TRUE)`: Excel 0, ours -1.2E-18). Multi-predictor
    exact fits: Excel's QR leaves no noise here, so the column order, pivoting or the reflector differs for p > 1.
    This is the next thing to score.
  - t4222 Calc!A32, B32: LOGEST standard errors (row 2), 1 ulp. They still come from (D'D)^-1. Port the se to R^-1
    (slopes: σ²·diag((R'R)^-1) on the centred design; intercept: σ²·(1/n + xm'(Xc'Xc)^-1 xm) is a candidate).
  - regression-trend O34, O35, N35 (x near 1E10, see the first notes). Scored: the prediction ym + Σ b_j·(x_j - xm_j)
    gives Excel's 3.6666666665000003 on O34, but over the whole eval it fixes 14 fewer cells than b0 + b·x (194 vs 208
    with the old stats rows). Not taken; a mixed rule needs evidence.
  - ironcalc-calc TREND_GROWTH.xlsx Sheet1!K5 (not looked at).
- Gate A (`rustsheet-eval accuracy`) with the QR stats: 20 of the 21 T-2006 known-failure lines are stale (inside the
  envelope), and only LOGEST/s1 A110 stays known. One new failing point, LINEST/s1 A184 (the intercept cancellation in
  the first notes), is still unresolved.
- Still owed before a PR: acceptance criteria 1 (fitlab `regression` kernel, candidate table in statistics.md), the R4
  hold-out and the fresh sweep, and replacing the `KERNEL` scaffold with one form (clippy has not been run on it).

Resumed 2026-11-07 06:02 UTC by an opus run (worker step 2, branch idle 5 h).

## Landing notes (2026-11-07 06:10 UTC, third opus run)
Short resume (the session's start-new-work limit was near): no code change. Data for the next scoring step, from
probes/linest-modes (oracle, excel-graph), for the multi-predictor exact-fit regressions L85, M85 and L106:
- Data: A2:A9 = 1..8, B = {2.1,3.9,6.2,7.8,10.1,12.2,13.8,16.1}, C = {2,1,4,3,6,5,8,7}, D = {3,5.5,9,16,27,50,88,160},
  E = 2·A exactly. y is B in both formulas, so y is itself an x column.
- K85 `LINEST(B2:B9,A2:C9,TRUE,TRUE)`: Excel is exact everywhere: C 0, B 1, A 0, intercept 0; row 86 se all 0; r2 1,
  se_y 0, F `#NUM!`, df 4 (no column dropped), ss_reg 167.79500000000004, ss_res 0.
- K106 `LINEST(B2:B9,A2:E9,TRUE,TRUE)`: E 1.4342828472961902E-16, D exactly 0, C 5.0756899759395145E-18, B
  0.9999999999999999, A exactly 0 with se exactly 0 (O107) and df 3: Excel drops A, the earlier of the collinear pair
  A, E, and keeps E. The other se (row 107) are 5.4E-19..8.5E-17, r2 1, se_y 3.67E-17, F 3.1E+34, ss_res 4.0E-33.
- Reasoning so far (not scored): when y equals an x column and that column is reflected last, every earlier
  reflector acts on y and the column with the same bits, so back substitution gives exactly 1 and 0. That explains
  K85 only if B is reflected after A and C (neither the natural nor the reversed order does that), and K106's noise on
  B and E says B is not last there. So a column order alone does not explain both: score pivoted QR (largest residual
  norm first, LINPACK dqrdc) and the drop of A rather than E (which column the collinearity test removes) next, with
  a replica in Python of `householder` (same summation order) on these two formulas before touching linest.rs.
- Scored after the notes above (06:00-06:20 UTC, branch merged with origin/main 2e192ce4, `rustsheet-eval check`):
  - KERNEL 9 as committed: fixed 321 (probes 279, ironcalc-calc 38, gen-m9 4), REGRESSIONS 9 (probes 8, ironcalc-calc 1).
  - A Python replica of `householder` (same summation order) over every column order, applied or -sign·norm
    diagonal, pivoting by largest residual norm, centred or not: K85 is exact (C 0, B 1, A 0) with the applied
    diagonal in the natural order (and with pivoting); K106 (A dropped, B..E) is matched by no candidate (at best 2
    of B, C, D, E).
  - KERNEL 9 with the applied diagonal in the engine: fixed 259 (probes 223, ironcalc-calc 32, gen-m9 4), REGRESSIONS 6.
    It trades 62 fixes for 3 regressions, so it is not taken. The merge commit of origin/main is kept (pushed).

Resumed 2026-11-07 12:05 UTC by an opus run (worker step 2, branch idle 6 h).

## Landing notes (2026-11-07 12:35 UTC, fourth opus run)
- **Committed: standard errors from the centred design under KERNEL 9 with an intercept.** Slope j is
  sqrt(se_y² · C⁻¹_jj), the intercept sqrt(se_y² · (1/n + xm'·C⁻¹·xm)), C = Xc'Xc summed row by row, inverted with
  `invert_matrix`; se_y² is the QR tail over df. Python replica on t4222 Calc!A32 `LOGEST({2,4,5,4,5},{1..5})` row 2:
  slope 0.08794233738872581 and intercept 0.29167173630524657, both Excel's bits. Rejected (each 1 ulp off on one of
  the two): sqrt(var)/|r11|, sqrt(var/r11²), sqrt(var)/sqrt(sxx), and for the intercept any form through R's
  r11 instead of sxx (sqrt(var·(1/n + xm²/r11²)), sqrt(var/r00² + (xm·se1)²)). The (D'D)⁻¹ form stays for const=FALSE.
- `make eval-check` on this branch (main as of the third run's merge): fixed 325 (probes 283, ironcalc-calc 38,
  gen-m9 4), REGRESSIONS 7: linest-modes L85, M85, L106 (multi-predictor exact fits, see the third notes),
  regression-trend N35, O34, O35 (x near 1E10), ironcalc-calc TREND_GROWTH Sheet1!K5. t4222 A32, B32 are fixed.
- Still owed before a PR: the 7 regressions above, acceptance criterion 1 (fitlab `regression` kernel and the
  candidate table in statistics.md), the R4 hold-out and fresh sweep, and replacing the `KERNEL` scaffold with one form.
