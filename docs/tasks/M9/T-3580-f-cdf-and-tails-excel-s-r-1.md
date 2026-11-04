---
id: T-3580
title: "F cdf and tails: Excel's r >= 1 incomplete-beta arguments, and bgrat at b = 1/2 for a df of 1"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
F.DIST(., TRUE), F.DIST.RT and FDIST hand the incomplete beta Excel's own (x, y), and F with a numerator or
denominator df of 1 takes bratio's bgrat branches at b = 1/2 as the Student t tails and BETA.DIST already do.

## Context
- Found by T-3567 (docs/research/t2464/r5-bgrat-fdf1/RESULTS.md; statistics.md "F's incomplete-beta arguments
  keep r >= 1"). With mx = fl(m·x) and the truncated dfs m, n, Excel forms r = fl(n/mx) when mx <= n, else
  r = fl(mx/n), so r >= 1, and passes the smaller argument as fl(1/fl(1 + r)) and the larger as fl(r/fl(1 + r)).
  Today `f_beta_arguments` (crates/rustsheet-engine/src/functions/statistical/fisher.rs) forms both as 1/(1 + .),
  which is right for the smaller one only.
- With those arguments, the T-2509 bgrat at b = 1/2 (`bratio::t_kernel_tails`' `half_bgrat`) fits F at a df of 1:
  the reason T-2510 kept F off it (`FDIST(2,1,9)` 7 ulp, the d1-1 / d2-1 sweep cells visibly off) was the
  argument form, not the kernel. `ported_tails_with` carries `TODO(T-3580)` for this.
- Research model (python, r2-tdist FINAL kernel with `bg = "ld"`): on the hold-out probe
  probes-numeric/t3567-r5-fdf1-hold (3,578 cells) it scores 2,787/3,419/3,463 (bit-exact / within 4 ulp / within 15
  digits) against the engine's 1,857/2,843/3,025; the d1 = 1 bgrat region 991/1,000/1,000 against 298/572/659.
  On cells.tsv's 3,435 F cdf/tail cells the r >= 1 form beats n + mx in every branch but bfrac (64 against 65 bit-exact, 138
  against 133 within 4 ulp), and the a0 = 1 / b0 = 1
  closed forms are 162/162 and 201/201 bit-exact.
- Where the engine's other F pieces (bpser, bup, bfrac) differ from the research kernel, keep whichever is closer
  to Excel per branch, measured on the hold-out; the argument form alone should not be assumed to help there.
- F.INV, F.INV.RT and FINV invert `f_quantile_tail` on today's kernel (T-2511, T-2977): out of scope here unless the
  gated quantile cells stay put.

## Acceptance criteria
- [x] F cdf and tails use the r >= 1 arguments; F at a df of 1 takes bgrat at b = 1/2.
- [x] On t3567-r5-fdf1-hold, the bg1 region (d1 = 1, bgrat) reaches at least 990 of 1,000 cells bit-exact, and the
      case as a whole has no cell newly visibly off.
- [ ] `FDIST(2,1,9)` (gen-m9/c18dbf21f11093f3 A151) and the T-2977 gated cells stay at Excel's value; no gated
      known failure appears (`make eval-baseline` only shrinks baselines). **Not met:** see the Outcome and decision 0263.
- [x] statistics.md's "F's incomplete-beta arguments keep r >= 1" entry gets its "Ours:" line.
- [x] The decision 0188 R4 guards for a structural model (reviewer-excel-fidelity check 7) are shown in the Outcome:
      the hold-out against the correctly rounded value (CR on t3567-r5-fdf1-hold: 1,407/2,484 of 3,578 bit-exact /
      within 4 ulp, against the research model's 2,787/3,419), a fresh sweep oracle-filled after the freeze, and the
      Gate A envelope count on it with every point outside listed.
- [x] `f_beta_arguments`' doc comment no longer claims the 197 closed cells for both arguments.
- [x] `make check` green; reviewers approve.

## Out of scope
- F.DIST(., FALSE) (the density, T-2586).
- The bgrat increment at b != 1/2 (T-3582).

## Outcome
- `fisher.rs` `f_beta_arguments` forms Excel's r >= 1 arguments (r = fl(n/mx) or fl(mx/n); smaller fl(1/fl(1 + r)),
  larger fl(r/fl(1 + r)); (0, 1) at x = 0), and F's tails run through the new `bratio::f_closed_or_ported_tails`, which
  takes bgrat at b = 1/2 and L140 bup+bgrat as the Student t and BETA.DIST do. `half_bgrat` stays: the binomial family
  and F.TEST still run without it. F.INV / F.INV.RT / FINV are untouched (`f_quantile_tail`). Two unit tests.
- Structural model (check 7 (c)); the guards (docs/research/t3580/RESULTS.md):
  - Hold-out t3567-r5-fdf1-hold (3,578): **2,879/3,430/3,487** (bit-exact / 4 ulp / 15 digits) against main
    1,984/2,850/3,029 and the correctly rounded 1,407/2,484; bg1 (d1 = 1, bgrat) **991/1,000/1,000**; 0 cells newly
    visibly off, 458 newly within 15 digits. Per branch the new arguments win or tie everywhere, so nothing else
    was changed.
  - R4.1 train against hold-out (margin 5 points of bit-exact rate, stated before scoring): train is cells.tsv's
    3,435 F cdf/tail cells, the set T-3567 chose the form on (`score_train.py`): 2,647/3,350/3,403 (77.1%), against
    the correctly rounded 1,497/2,972/3,165; hold-out 80.5%, within the margin, and above CR there.
  - Fresh sweep probes-numeric t3580-f-fresh (2,400 cells, drawn and excel-graph-filled after the freeze at 3f4e01a2):
    1,867/2,280/2,335 against main 1,278/1,924/2,040 and the correctly rounded 932/1,718/1,872; 0 newly visibly off.
  - Gate A envelope: 2,400/2,400 fresh-sweep cells inside (none to list; math attached as
    `t3580-f-fresh.math50.json`); new batch `eval/accuracy/tables/F.DIST/r4-T-3580` 196/196 inside;
    `rustsheet-eval accuracy` 13,334 points, 0 failing.
- **Not met as written:** `FDIST(2,1,9)` (gen-m9/c18dbf21f11093f3 A151 and 7c20351562a2d71d A205) moves 1 ulp off
  Excel, and `F.DIST.RT(1,7,9)` (a790a4d4a6a82756 A121) 2 ulp. Both are bup's arithmetic on Excel's arguments
  (bup+bgrat at b = 1/2, L140 bup+bpser), which T-3567 left open; keeping them would be a keep-matched branch that
  loses on the hold-out (65 against 31 and 220 against 204 bit-exact). Decision 0263 allows exactly these three
  baseline lines. The other T-2977 gated cells hold.
- Eval delta: gen-m9 known failures 1,954 -> 1,939 (fixed 18, +3 under decision 0263), probes 4,433 -> 4,413 (fixed 20),
  other suites unchanged; visible parity 2,310 in-scope cells visibly off (lock 2,404).
- statistics.md's entry gets its `Ours:` line, the bup misses and the fresh-sweep counts.
