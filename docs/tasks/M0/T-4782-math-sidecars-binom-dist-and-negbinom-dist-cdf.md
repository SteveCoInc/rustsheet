---
id: T-4782
title: "Math sidecars: BINOM.DIST and NEGBINOM.DIST cdf at a large n through the converging beta tail (508 cells skipped as not converged)"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every BINOM.DIST and NEGBINOM.DIST cumulative cell that the math sidecars now skip as "not converged" gets its 50-digit
math value. The binomial cdf at a large n and a small p, and the negative binomial cdf at a large failure count, then
have a reference, so decision 0188 R1(c) stops counting those cells for having no math value.

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main b9ae5216, every suite fetched, excel-graph). `make unowned-off ALL=1`
  lists 17 visibly-off cells with no open owner, all in probes/t3444-binom-range-inv-large-n. T-4773 attaches the math
  for those 17 (including Range!B49 and B53, two of the not-converged skips below). This task is the split that
  covers every other not-converged BINOM.DIST/NEGBINOM.DIST skip: the definition in discrete.py, and the other
  sidecars.
- 508 BINOM.DIST and NEGBINOM.DIST cdf cells in 7 sidecars are under `skipped` with `why: not converged`:

  | case | skipped |
  |---|---|
  | probes-numeric/t3960-binom-negbinom-small-p-cdf | 140 |
  | probes/t3444-binom-range-inv-large-n | 89 |
  | probes/t3352-binom-negbinom-large-n | 75 |
  | probes-numeric/t3960-binom-negbinom-small-p-cdf-hold | 69 |
  | probes/t3740-binom-range-past-mean-sweep | 67 |
  | probes-numeric/t3352-binom-negbinom-sweep | 65 |
  | probes/t3576-negbinom-cdf-huge-s | 3 |

- Root cause: tools/src/rustsheet_tools/accuracy/defs/discrete.py writes `binom_cdf` as
  `mpmath.betainc(n-k, k+1, 0, 1-p)` and `negbinom_cdf` as `mpmath.betainc(s, f+1, 0, p)`. mpmath evaluates these
  through `hyp2f1` with the large parameter on the slow side of the series. At n = 1E6, p = 0.001 it raises "hypsum()
  failed to converge ... using a working precision of 7181 bits", so `tables.py` drops the point. The other tail
  converges in well under a second at dps 60 and 80, and both precisions agree to 25 digits (triage scratch run,
  mpmath in tools/.venv):
  - `BINOM.DIST(999,1000000,0.001,TRUE)` as `1 - betainc(k+1, n-k, 0, p)`: 0.4957884448330077066523817. A direct pmf
    sum at 40 digits gives 0.49578844483300770665.
  - `BINOM.DIST(526,600000,0.001,TRUE)`: 0.001107528039888733792235267 (probes/t3352-binom-negbinom-large-n Calc!A52:
    Excel 0.001107528039888131, engine 0.0011075280398886653).
  - `NEGBINOM.DIST(1041120,4940,0.00472551518826802,TRUE)` as `1 - betainc(f+1, s, 0, 1-p)`:
    0.5199426990785873165819 (sweep A184: Excel 0.5199426990783227, engine 0.5199426990785869). The direct form fails
    at dps 60 and 80.
  - `NEGBINOM.DIST(3827391,6017,0.00159874266118577,TRUE)`: 0.9244662311053954474342 (sweep A208).
  The complement loses digits when the cdf is tiny: 1 - x cancels at about log10(1/cdf) digits. So the switch needs a
  guard, either a higher dps for the complement or keeping the direct form where the cdf is small. `binom_sf` and
  `negbinom_sf` already take the converging side for the upper tail, and they are the pattern to follow. T-3960
  worked around the same skips in a scratch scorer: an outward pmf sum at 45 digits, and NEGBINOM.DIST as
  1 - BINOM.DIST(r - 1, f + r, p) (`docs/research/t3960/score.py`, statistics.md "BINOM.DIST and NEGBINOM.DIST at a
  large n"). That scorer checks itself against the sidecar to 1E-12 relative, so it can cross-check the new values.
- Cells R1(c) counts today only because they have no math value (visibly off, attributed, `r1c_left_out` empty).
  Decision 0325: this task names them, and their fit owners keep them:
  - probes/t3352-binom-negbinom-large-n.case.json (1): Calc!A52 (owner T-3961)
  - probes-numeric/t3352-binom-negbinom-sweep.case.json (2): Calc!A184, A208 (owner T-3961)
  - probes-numeric/t3960-binom-negbinom-small-p-cdf.case.json (34): Calc!A8, A29, A31, A35, A37, A44, A62, A68, A94,
    A97, A106, A109, A126, A135, A201, A207, A229, A233, A268, A300, A303, A324, A342, A381, A382, A404, A405, A406,
    A426, A428, A429, A430, A436, A437 (owner T-3961)
  - probes-numeric/t3960-binom-negbinom-small-p-cdf-hold.case.json (17): Calc!A1, A3, A23, A46, A47, A54, A55, A66,
    A68, A101, A121, A141, A152, A165, A177, A182, A194 (owner T-3961)
- T-3961's "where a math value exists" table has math for only 3 of its 11 cells, and its p' hypothesis was scored on
  those. The values this task adds are the reference T-3961's candidates are ranked against. Gate A's BINOM.DIST
  table (T-3741) is not changed by this task. If its points pass through the same definition, run its check too.
- Regenerate case by case (`rustsheet-accuracy math --function`, T-4483), never the whole suite. t3444 takes about
  40 minutes at n 1E9 to 2^31-2 (T-3741, T-3993 Outcome), so run it alone in the background. A killed
  `ProcessPoolExecutor` leaves spawn workers behind (T-3993).

## Acceptance criteria
- [ ] `binom_cdf` and `negbinom_cdf` take a form that converges at the skipped points and keeps 50 digits where the
      cdf is tiny. Unit tests in tools/tests compare it against an independent route (a direct pmf sum at n ≤ 1E7, or
      the other tail at a raised dps) at a large-n small-p point, a tiny-cdf point and a NEGBINOM point at f ≈ 1E6.
- [ ] The 7 sidecars above are regenerated. Each "not converged" BINOM.DIST/NEGBINOM.DIST skip is gone or gets a
      stated reason in the Outcome. `make eval-math-check` is current.
- [ ] The Outcome gives the decision 0188 R1(c) counts before and after (probes and probes-numeric: counted, Phase 1b
      evidence, R3 passing) and says, for each of the 54 cells above, whether it became Phase 1b evidence or stays
      counted.
- [ ] No eval/baseline file changes (math values do not score cells).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes, and any fit of Excel's cdf (T-3961, T-3959, T-3774, T-4746).
- The 17 t3444 cells T-4773 owns (Inv!C and Inv!D, whose argument reads a BINOM.INV formula cell, and Range!B49,
  B53). If T-4773 lands a convergent BINOM.DIST route first, reuse it here rather than adding a second one
  (directive 3); if this task lands first, T-4773 reuses this one.
