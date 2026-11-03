---
id: T-3953
title: "Oracle sweep: F.DIST.RT and FDIST far right tail on bfrac (df1 >= 80, df2 <= 200, kernel x0 below 1E-6), with a hold-out, for the FDIST(1E+15,4490,6) fit"
milestone: M9
lane: research
status: in-progress
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
- [ ] A generator under docs/research/t3953/ (seeded, committed) writes probes-numeric `t3953-f-tail-bfrac-tiny`:
      about 1,200 rows, d1 log-uniform in [80, 20000] (integers and some non-integers, which Excel truncates), d2
      log-uniform in [1, 200], X chosen so that x0 = d2/(d1·X + d2) is log-uniform in [1E-280, 1E-6], keeping only rows
      where the engine's dispatch is `Branch::Bfrac` and the tail is a normal double above 1E-300. Columns per row:
      `F.DIST.RT(X,d1,d2)`, `FDIST(X,d1,d2)`, `BETA.DIST(v,d2/2,d1/2,TRUE)` with v computed in the sheet as
      `d2/(d1*X+d2)`, and the same BETA.DIST at v given as an exact literal, to separate the argument form from the
      kernel.
- [ ] Before any scoring, about 30% of the rows by row hash are marked hold-out in the generator (decision 0188 R4.1),
      and the split is recorded in the case description.
- [ ] Filled from excel-graph and canary-verified; a second fill matches the first (decision 0112), or the cells that
      differ are listed and dropped as targets.
- [ ] A math sidecar from `rustsheet-accuracy math` (50 digits) is committed for the case, and the Outcome gives Excel's
      and the engine's exact / 4 ulp / 15-digit counts against Excel and Excel's error distribution against the math.
- [ ] Every visibly-off cell of the new case has an open owner (T-3954 lists the case), so `new_case_owners.py` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3954), bfrac's Lentz region near the mean (T-3913), basym (T-3597).
