---
id: T-3953
title: "Oracle sweep: F.DIST.RT and FDIST far right tail on bfrac (df1 >= 80, df2 <= 200, kernel x0 below 1E-6), with a hold-out, for the FDIST(1E+15,4490,6) fit"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An excel-graph probes-numeric case gives Excel's F right tail where bratio's dispatch is bfrac at a tiny kernel argument,
with a hold-out fixed before any scoring, so T-3954 can find Excel's arithmetic there without fitting blind.

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main c2062e5c8). gen-m9/8f2ac25945af87da Calc!A92
  `=FDIST(Data!$D7,DAYS360(31271,35828,AND(TRUE,TRUE)),6)` = `FDIST(1E+15,4490,6)` is visibly off and owned only by
  deferred T-2221. Excel is 1.18E-14 relative off the 50-digit math and ours 4.6E-15 (T-3954 has the numbers), so the
  fix is Excel's own arithmetic, and one cell cannot choose it (decision 0056, decision 0188 R4).
- The right tail is I_v(d2/2, d1/2) at v = d2/(d1·X + d2) (`fisher.rs` `f_beta_arguments`). At this cell a0 = 3,
  b0 = 2245, x0 = v = 1.336E-18, no swap (λ > 0), b0 >= 40 and a0 <= 100, so `bratio::dispatch` gives `Branch::Bfrac`
  (`crates/rustsheet-engine/src/functions/statistical/bratio/mod.rs`). No probe covers that corner: T-3913 probes
  bfrac's Lentz region near the mean (z in [-8, -0.2] sd), and the T-2220 sweeps keep df2 >= 1 with moderate X.
- No cached workbook has more than a handful of F tail cells at x0 < 1E-6 with df1 >= 80 (the gen-m9 sweep has this
  one), hence this sweep (decision 0031; the excel-probe skill).

## Acceptance criteria
- [x] A generator under docs/research/t3953/ (seeded, committed) writes probes-numeric `t3953-f-tail-bfrac-tiny`:
      about 1,200 rows, d1 log-uniform in [80, 20000] (integers and some non-integers, which Excel truncates), d2
      log-uniform in [1, 200], X chosen so that x0 = d2/(d1·X + d2) is log-uniform in [1E-280, 1E-6], keeping only rows
      where the engine's dispatch is `Branch::Bfrac` and the tail is a normal double above 1E-300. Columns per row:
      `F.DIST.RT(X,d1,d2)`, `FDIST(X,d1,d2)`, `BETA.DIST(v,d2/2,d1/2,TRUE)` with v computed in the sheet as
      `d2/(d1*X+d2)`, and the same BETA.DIST at v given as an exact literal, to separate the argument form from the
      kernel.
- [x] Before any scoring, about 30% of the rows by row hash are marked hold-out in the generator (decision 0188 R4.1),
      and the split is recorded in the case description.
- [x] Filled from excel-graph and canary-verified; a second fill matches the first (decision 0112), or the cells that
      differ are listed and dropped as targets.
- [x] A math sidecar from `rustsheet-accuracy math` (50 digits) is committed for the case, and the Outcome gives Excel's
      and the engine's exact / 4 ulp / 15-digit counts against Excel and Excel's error distribution against the math.
- [x] Every visibly-off cell of the new case has an open owner (T-3954 lists the case), so `new_case_owners.py` passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3954), bfrac's Lentz region near the mean (T-3913), basym (T-3597).

## Outcome
- Added probes-numeric `t3953-f-tail-bfrac-tiny`: 1,200 rows × 4 columns (A F.DIST.RT, B FDIST, C BETA.DIST with v
  formed in the sheet, D BETA.DIST at the engine's x0 as a 15-digit literal), generator docs/research/t3953/mksweep.py
  (seed 3953; 7,490 draws rejected as not bfrac, a duplicate, or a tail at or below 1E-300).
- Hold-out (decision 0188 R4.1): the fit lab's mandatory per-cell split (`fitlab/holdout.py`), on column A only.
  B always equals A and C equals A on 1,015 rows, so a per-cell split over every column would put copies of a cell on
  both sides; T-3954 fits A and keeps B, C and D as diagnostics. 369 A cells are hold-out (831 tune), marked in the
  row's hypotheses and the case description; the accept margin is the fit lab's default, 5 percentage points, stated
  before any scoring of a candidate (the counts below score today's engine only).
- Deviations: d2 is integer only (the task asked non-integers of d1 only); a row with TRUNC(d2) <= 2 is never bfrac
  (a shape <= 1), so d2 starts at 3 in practice. Literal v has 15 significant digits, the most a formula literal
  carries (numbers.md), so column D is "the engine's x0 to 15 digits", not the exact double, and does not test the
  kernel at an exact v.
- Filled from excel-graph (canary-verified by the fill); a second uncached recalc matched on all 4,800 cells.
- Math sidecar (`rustsheet-accuracy math`) committed; it covers C and D only, since the accuracy tool defines no
  F.DIST.RT / FDIST. docs/research/t3953/score.py scores A and B against the row's C math (the same exact quantity);
  every figure here is in docs/research/t3953/score.out.
- F.DIST.RT, engine against Excel (all / tune / hold): exact 4/2/2, 4 ulp 82/61/21, 15 digits 189/135/54. FDIST is
  identical. Against the correctly rounded math, Excel: exact 8, 4 ulp 75, 15 digits 180; engine: 13, 87, 235.
- Excel's relative error against the math (F.DIST.RT): median 1.1E-14, p90 4.4E-14, p99 1.1E-13, max 1.5E-13, 623 high
  and 577 low (column D's max is 1.6E-13); ours median 6.5E-15, max 1.1E-13, 574 high and 626 low. None above 1E-4
  (no R3 case).
- Recorded in statistics.md, "F right tail on bfrac at a tiny kernel argument".
- Visibly-off cells (1,011 per F column) are all owned by T-3954, which names the whole case. Eval delta: none
  (probes-numeric is ungated).
