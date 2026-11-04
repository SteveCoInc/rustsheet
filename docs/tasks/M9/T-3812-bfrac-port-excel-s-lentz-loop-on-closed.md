---
id: T-3812
title: "bfrac: port Excel's Lentz loop on closed-form terms (T-3571 model)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The engine's bfrac reproduces Excel's arithmetic: TOMS 708's continued fraction run through Numerical Recipes' modified
Lentz loop on closed-form terms, with Excel's λ, as docs/research/t2464/r5-bfrac/model.py freezes it.

## Context
- Decision 0188 R5 does not defer this port: its cells are not all within 15 digits of Excel today. On
  t3571-r5-bfrac-fresh the TOMS recurrence the engine runs is within 15 digits on 844 of 1,200 points, the model on
  1,188 (decision 0114's test), so the port is Gate A-level work, not last-digit reproduction.
- T-3571 (docs/research/t2464/r5-bfrac/RESULTS.md, statistics.md "Incomplete beta bfrac is TOMS's fraction in a Lentz
  loop"): on Excel's own density the model scores 1,158/1,630/1,632 of 1,632 on probes-numeric t3571-r5-bfrac-hold and
  694/1,177/1,188 of 1,200 on t3571-r5-bfrac-fresh, against 162/977/1,384 and 86/532/844 for the TOMS recurrence the
  engine runs today (`crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs`, T-2508).
- The model, line by line: `model.py` `fraction` and `cdf`. λ = fl(a - fl((a+b)x)) in the unswapped call for a > b
  as well (TOMS takes (a+b)y - b there).
- The evidence covers only a, b in [40, 5000] on the unswapped call (x below the mean, z from -8 to -0.2 sd). The Lentz model is a structural choice plus one fit: EPS = 10·DBL_EPSILON, tagged
  `/// FITTED(T-3571, 1536 cells, a, b 40..5000, unswapped call (x below the mean), z -8..-0.2 sd)` at the constant (decision 0188 R4.3). Gate the whole Lentz path to that region; everywhere else in bfrac's
  region (a0 < 40 with b0 >= 40, a or b above 5000, the swapped call, within 0.2 sd of the mean) keep today's TOMS
  recurrence, the accurate path, and say so in a comment. TINY = 1e-30 is NR's FPMIN (not a fit).
- NR's loop replaces D or C by FPMIN when its magnitude falls below FPMIN. model.py omits these guards and nothing on
  the probed region depends on them; the port keeps them so the engine cannot divide by zero (directive 5). That is a
  safety choice, not a claim from evidence.
- The engine's density (T-3244) is not Excel's at large shapes, so engine bit-exact counts stay below the model's
  (which reads Excel's density off the sheet); score both with the engine's density and record them.

## Acceptance criteria
- [x] `bfrac.rs` evaluates r with the Lentz loop, b0 = c·(a/(a+1)), α = v/s/s·u·n·(b-n)·x·x, β = n + fl(n·x·(b-n))/s +
      ((a+n)/(a+2n+1))·(c + n·(y+1)), stop |C·D - 1| <= 10·DBL_EPSILON, cdf = fl(brc·f), on the gated region only, with the FITTED tag
      line above and NR's FPMIN guards on D and C kept; the term bound and `None` path stay (directive 5); outside the region the TOMS recurrence is unchanged.
- [x] Unit tests pin a few cells of t3571-r5-bfrac-hold where the engine's density equals Excel's.
- [x] Eval: probes-numeric t3571-r5-bfrac-* and t2464-w2f-xruns-1's bfrac cells counted before and after in the
      Outcome; no eval regressions (or a decision record per decision 0082 for cells lost to the better model).
- [x] Gate A envelope rechecked with the engine's density on probes-numeric t3571-r5-bfrac-fresh (T-3571's 1,200 of
      1,200 used Excel's density read off the sheet), every point outside listed; once BETA.DIST has a Gate A table
      (T-3529), an `r4-T-3812` batch in `eval/accuracy/tables/BETA.DIST/` instead (decision 0188 R4.2).
- [x] `make check` green; reviewers approve.

## Out of scope
- The open per-term rounding past 10 terms (T-3813); the density itself (T-3244); basym (T-3322).

## Handoff (resumed)
Sonnet run (no code written): handed off before starting because the task is a numeric fit under decisions 0056 and
0188 R4 (FITTED tag, hold-out split, fresh sweep, Gate A envelope recheck with the engine's density). Next steps: read
docs/research/t2464/r5-bfrac/model.py and RESULTS.md, port the Lentz loop in `bratio/bfrac.rs` gated to a, b in
[40, 5000] on the unswapped call, keep the TOMS recurrence elsewhere and the FPMIN guards, then score the probes-numeric
t3571-r5-bfrac-* cells before and after as the acceptance criteria say.

## Outcome
- `bratio/bfrac.rs`: `lentz_fraction` is T-3571's `model.py` `fraction` (NR's modified Lentz loop from TINY = 1e-30,
  b0 = c·(a/(a+1)), α = v/s/s·u·n·(b-n)·x·x, β = n + fl(n·x·(b-n))/s + ((a+n)/(a+2n+1))·(c + n·(y+1)), λ =
  fl(a - fl((a+b)x)), stop |C·D - 1| <= `LENTZ_EPS` = 10·DBL_EPSILON with the FITTED tag line of the task), cdf =
  fl(brc·f). NR's FPMIN floors on D and C are kept (directive 5); the `BFRAC_MAX_TERMS` bound and `None` path stay.
  `lentz_region` gates it to the unswapped call with a, b in [40, 5000] and z = (x - mu)/sd in [-8, -0.2] (mkprobe.py's
  mu and sd); everywhere else TOMS's recurrence is unchanged. `bfrac` now takes the `Dispatch`. `quality/fitted.json`
  updated with `check_fitted.py --update` (bfrac.rs 0 -> 1; it also recorded gamma.rs 7 -> 6, main's own drift), under
  decision 0257.
- Port check: given the same density, the engine's cdf equals `model.py` `cdf` bit for bit on all 1,536 tune, 1,632
  hold and 1,200 fresh points; `model.py` on Excel's density reproduces RESULTS.md's 1,236, 1,158 and 694 bit-exact.
- Hold-out and fresh sweep: T-3571's (split fixed in 65189a46d before scoring; fresh sweep built after the freeze);
  this task adds no new fit.
- Scores on the cdf cells with the engine's density (bit-exact / within 4 ulp / within half a unit of the 15th digit),
  before -> after: tune 44/466/1,055 -> 63/578/1,069 of 1,536; hold 45/428/913 -> 102/635/978 of 1,632; fresh
  18/175/431 -> 39/252/490 of 1,200. The engine's density equals Excel's on only 51, 81 and 32 of those points
  (T-3244), so the counts stay far below the model's.
- t2464-w2f-xruns-1: 865 cdf cells of the (100, 120) run are on the region: 43/325/480 -> 32/351/496 (whole case
  3,942/7,375/7,774 -> 3,931/7,401/7,804 of 8,192). The 11 lost bit-exact cells were TOMS matches against an engine
  density 5 ulp off Excel's: e.g. A14913 (density A14914 5 ulp off) is TOMS exact, Lentz 7 ulp; with Excel's density the
  model is 1 ulp off there. bratio/tests.rs `bfrac_matches_excel` pinned A14913 and A15779: they move to the comment,
  and `bfrac_lentz_matches_excel` pins hold A700, A1591, tune A133 and fresh A470, where the engine's density equals
  Excel's (each off before). `ported_tails_are_swap_symmetric` skips Lentz-region points (only the unswapped call takes
  Excel's loop), 607 -> 593 compared.
- Gate A envelope with the engine's density on t3571-r5-bfrac-fresh: 1,200 of 1,200 inside (475 within 15 digits of
  Excel; the rest within 1e-9 of the math sidecar, and 4 points the sidecar has no value for, A239, A441, A902, A978,
  are 1.1e-14 to 2.1e-14 from a 60-digit continued fraction). Tune and hold: 1,536 and 1,632 inside. BETA.DIST has no
  Gate A table yet (T-3529).
- Eval: `make eval-check` REGRESSIONS 0 in every suite, fixed 0 (the t3571 and xruns probes-numeric cases are not
  gated). statistics.md "Incomplete beta bfrac is TOMS's fraction in a Lentz loop" Ours line updated. Decision 0257
  records the fitted lock and accepts the 11 lost ungated xruns-1 cells and the 2 removed test pins (round 2's
  excel-fidelity finding).
- Follow-up filed: T-3913 (F.DIST and BINOM.DIST on the Lentz region, round 2's MINOR).
