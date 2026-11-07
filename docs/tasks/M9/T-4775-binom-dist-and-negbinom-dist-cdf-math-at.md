---
id: T-4775
title: "BINOM.DIST and NEGBINOM.DIST cdf math at a huge n: the 54 counted cells rustsheet-accuracy drops as not converged"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math` gives a 50-digit value for BINOM.DIST and NEGBINOM.DIST cumulative cells at a huge trial or
failure count (n or number_f from about 1E5 up to 2^31 - 2, and number_f past 1E8 for NEGBINOM.DIST). Today it drops
them as `not converged`. Once they have a math value, decision 0188 R1(c) can sort the 54 visibly-off cells below
(as Phase 1b evidence where both sides are within 1e-9 of the math). Until then each one counts in the M9 probe row,
because a cell with no math value counts (decision 0325).

## Context
- Gap triage 2026-11-07 (`make eval` on agent/functions/tasks-opus-refill-0307 at b9ae5216f, excel-graph). M9's probe
  row counts 816 R1(c) cells. 54 of them are BINOM.DIST or NEGBINOM.DIST cdf cells that the math sidecar lists under
  `skipped` with `why: not converged`.
- Root cause (checked): `tools/src/rustsheet_tools/accuracy/defs/discrete.py` `binom_cdf` and `negbinom_cdf` call
  `mpmath.betainc(n - k, k + 1, 0, 1 - p, regularized=True)` and `mpmath.betainc(s, f + 1, 0, p, ...)` directly.
  At these shapes mpmath's hypergeometric series fails to converge: on `BINOM.DIST(95,100000,0.001,TRUE)`,
  `mpmath.betainc` raises "hypsum() failed to converge to the requested 213 bits of accuracy using a working
  precision of 7248 bits". The definition then drops the point (dps 60 and 80 disagree, `tables.py`). `beta.py`'s
  `_betainc` already has a budgeted series/regime dispatch for BETA.DIST (T-3680), but the discrete definitions
  do not use it.
- Evidence that the cells have a well-defined math value and where both sides stand (triage scratch: a 60-digit sum
  of the pmf terms by the exact recurrence, k + 1 terms; p is the decimal literal, which differs from the double by
  less than 3e-14 relative in the cdf here):

  | cell | formula | Excel - math (rel) | ours - math (rel) |
  |---|---|---|---|
  | probes/t3352-binom-negbinom-large-n Calc!A52 | `BINOM.DIST(526,600000,0.001,TRUE)` | -5.5e-13 | -6.4e-14 |
  | probes-numeric/t3960-binom-negbinom-small-p-cdf-hold Calc!A1 | `BINOM.DIST(333,2959211,0.000121692473175055,TRUE)` | +4.3e-12 | -1.4e-12 |
  | t3960-...-hold Calc!A23 | `BINOM.DIST(227,14127647,1.67420715099773E-05,TRUE)` | +5.3e-11 | +3.1e-11 |
  | t3960-...-hold Calc!A55 | `BINOM.DIST(221,14898332,1.74518254167383E-05,TRUE)` | -2.7e-11 | -1.2e-10 |
  | probes/t3444-binom-range-inv-large-n Range!B49 | `BINOM.DIST(1100,1E6,0.001,TRUE)-BINOM.DIST(999,1E6,0.001,TRUE)` | +1.2e-12 | -2.5e-16 |

  So every checked cell is within 1e-9 on both sides. The engine is the worse side on some of them (A55 1.2e-10, A54
  3.4e-11). That is Phase 1b work, not this task's.
- Method choice is the open design question, hence opus. Direct summation from 0 is k + 1 terms, which is fine for
  BINOM.DIST at k up to about 1E5 but not for NEGBINOM.DIST at number_f 1.65E8 (t3960 Calc!A201) or BINOM.DIST at
  k near 5E8 (t3444's 1E9-trial rows). Calling `beta.py`'s `_betainc` as it stands is not enough either: its
  `_tail_series` budget (SERIES_MAXTERMS 1E5, about sqrt(370 a / (1 - x)) terms) is far short at a = n - k = 1E6 and
  1 - x = p = 1E-4, and above SERIES_SHAPE_CAP = 1E7 it falls back to mpmath alone, which is the failure seen here.
  Candidates: summation of the pmf outward from the mode (or from the near end) with a 50-digit stop bound and the
  pmf in log space, which suits a discrete cdf at any n; or the smaller-tail side I_p(k + 1, n - k) through
  `_betainc`, where x = p is tiny. The budget must stay deterministic (never the clock, T-3680), and the result must
  agree at dps 60 and 80.
- Scope, so this task merges independently of T-4776: change `defs/discrete.py` only. You may call `_betainc`, but do
  not modify it. T-4776 extends `_betainc` above SERIES_SHAPE_CAP for the beta quantiles. If T-4776 lands first and
  its regime covers these cells, calling it is enough. T-4036 (sonnet) tidies `_betainc` without changing its values.
- Owners of the cells' fix work, which keep them: T-3961 (Excel's cdf at p below 0.01 and a huge n, visible stage)
  lists the t3960 and t3352 cells, and T-3959, T-3774 and T-3198 name the t3352 cases. This task only attaches the math.
  If both sides then sit within 1e-9 (as the table suggests), the cells become Phase 1b evidence, and T-3961's
  visible stage should be re-read against decision 0325. Record that in the Outcome; do not edit T-3961 here.
  probes/t3444-binom-range-inv-large-n Range!B49 and B53 are T-4773's (filed in parallel); this task's method should
  make them converge too, but T-4773 attaches their math.

Cells this task owns for their math value (shared with T-3961, T-3959, T-3774 and T-3198 as above):
- probes/t3352-binom-negbinom-large-n.case.json (1): Calc!A52
- probes-numeric/t3352-binom-negbinom-sweep.case.json (2): Calc!A184, Calc!A208
- probes-numeric/t3960-binom-negbinom-small-p-cdf-hold.case.json (17): Calc!A1, Calc!A3, Calc!A23, Calc!A46, Calc!A47, Calc!A54, Calc!A55, Calc!A66, Calc!A68, Calc!A101, Calc!A121, Calc!A141, Calc!A152, Calc!A165, Calc!A177, Calc!A182, Calc!A194
- probes-numeric/t3960-binom-negbinom-small-p-cdf.case.json (34): Calc!A8, Calc!A29, Calc!A31, Calc!A35, Calc!A37, Calc!A44, Calc!A62, Calc!A68, Calc!A94, Calc!A97, Calc!A106, Calc!A109, Calc!A126, Calc!A135, Calc!A201, Calc!A207, Calc!A229, Calc!A233, Calc!A268, Calc!A300, Calc!A303, Calc!A324, Calc!A342, Calc!A381, Calc!A382, Calc!A404, Calc!A405, Calc!A406, Calc!A426, Calc!A428, Calc!A429, Calc!A430, Calc!A436, Calc!A437

The t3444 sidecar also drops 87 other Range cells as not converged (n up to 2^31 - 2, p 0.3 and 0.5). They are not
visibly off today, but the same change should give them values.

## Acceptance criteria
- [ ] `binom_cdf` and `negbinom_cdf` (and the `_sf` forms, if they share the failure) return a value that agrees at
      dps 60 and 80 for every cell above and for the t3444 Range cells now skipped as `not converged`. A unit test pins
      at least four of them against an independent 60-digit reference: the pmf-sum for a small k, and a second method
      for NEGBINOM.DIST at number_f 1.65E8.
- [ ] `cd tools && uv run rustsheet-accuracy math --suite probes` and `--suite probes-numeric` regenerate the four
      sidecars, and none of the 54 cells is under `skipped`. `rustsheet-accuracy math --check` passes.
      `rustsheet-accuracy verify BINOM.DIST` and `verify NEGBINOM.DIST` show no change on the existing Gate A
      tables (accuracy/BINOM.DIST, accuracy/NEGBINOM.DIST), or each change is explained in the Outcome.
- [ ] After `make eval`, each of the 54 cells is Phase 1b evidence in `report.json` `r1c_left_out` or is listed in the
      Outcome with its Excel-minus-math and ours-minus-math. The M9 probe row's R1(c) count drops by the Phase 1b
      ones. `eval/baseline/` gains no new line (this task changes no engine value).
- [ ] The point budget stays deterministic (no clock), and `make check` is green; reviewers approve.

## Out of scope
- Any engine change to BINOM.DIST or NEGBINOM.DIST (T-3961, T-3959, T-3774, Phase 1b).
- The t3444 Inv cells whose argument is a BINOM.INV formula cell (T-4777).
