---
id: T-3961
title: "BINOM.DIST and NEGBINOM.DIST cdf at p below 0.01 and a huge n: Excel is 269 to 64,789 ulp off the math where ours is close (11 visibly-off t3352 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3960]
campaign: visible
---

## Goal
The 11 cumulative BINOM.DIST and NEGBINOM.DIST cells below (p below 0.01, n or the failure count from 1E5 to 1E8)
are within 15 significant digits of Excel, from a model of how Excel forms or evaluates its cdf there. If no model is
found, the cells are an UNVERIFIED statistics.md entry with the hypotheses ruled out.

## Context
- Gap triage 2026-11-03b (`make eval` on origin/main c2062e5c8, excel-graph). Split from T-3774, which is blocked
  (time-boxed on the pmf; its cdf cells were never attempted), together with T-3959. T-3959 takes the other cdf
  cells. These 11 are a separate regime. Ours minus Excel is hundreds to tens of thousands of ulp, always positive
  (Excel is lower). Where a math value exists, Excel is far off it and ours is close:

  | cell | formula | Excel | ours | ours - Excel (ulp) | Excel - math, ours - math (ulp) |
  |---|---|---|---|---|---|
  | large-n A34 | `BINOM.DIST(70,100000,0.001,TRUE)` | 0.0009667927349360346 | ...61112 | +706 | -961, -255 |
  | large-n A52 | `BINOM.DIST(526,600000,0.001,TRUE)` | 0.001107528039888131 | ...886653 | +2464 | |
  | large-n A70 | `BINOM.DIST(905,1000000,0.001,TRUE)` | 0.0012086874997822498 | ...26675 | +1926 | |
  | large-n A88 | `BINOM.DIST(9700,10000000,0.001,TRUE)` | 0.0013064181746120147 | ...2302 | +1325 | |
  | sweep A132 | `BINOM.DIST(2461,16194261,0.000154374041567456,TRUE)` | 0.22119048320161896 | 0.22119048320341644 | +64761 | -64789, -28 |
  | sweep A147 | `BINOM.DIST(5887,623666,0.00947288369372644,TRUE)` | 0.39554725622481485 | ...86125 | +836 | |
  | sweep A156 | `BINOM.DIST(8259,868228,0.00961856313687781,TRUE)` | 0.15689278088052766 | ...53513 | +269 | |
  | sweep A171 | `NEGBINOM.DIST(106931620,1903,1.81058846428433e-05,TRUE)` | 0.7771607360384137 | 0.7771607360422212 | +34295 | -34302, -7 |
  | sweep A184 | `NEGBINOM.DIST(1041120,4940,0.00472551518826802,TRUE)` | 0.5199426990783227 | ...85869 | +2380 | |
  | sweep A190 | `NEGBINOM.DIST(544583,45,0.000105461509284258,TRUE)` | 0.9603825121576379 | ...76842 | +417 | -417, 0 |
  | sweep A208 | `NEGBINOM.DIST(3827391,6017,0.00159874266118577,TRUE)` | 0.924466231105322 | ...5395 | +658 | |

  Math values are the report's math sidecar (50 digits) and, for A34, mpmath `betainc` at 60 digits. Excel's worst
  relative error is 8.1E-12 (A132). That is far above basym's 100·eps stop and bfrac's 15·eps, so Excel's own branch
  arithmetic does not explain it. Something larger is at work: argument formation, or a different route at small p.
- Lead (UNVERIFIED, gap triage): the error grows with n·p·(the cdf's sensitivity to p), the size an error in the low
  bits of p would give. One test rebuilt p from the complement, p' = 1 - fl(1 - p), and took the math at p':
  - Excel moved closer to the math on large-n A34 (-961 to -714 ulp), A52 (-2780 to -2425), A70 (-2383 to -1884) and
    sweep A190 (-417 to -247);
  - it moved much further on NEGBINOM.DIST at small r (large-n A146 -9 to +373; sweep A171 -34302 to +287014).
  So "Excel uses p' throughout" is ruled out. A mixed formation (x from fl(1 - p) and y from p, inside λ or the
  exponent of basym or bgrat) is untested.
- Needs an oracle sweep. The cached workbooks have only these 11 points in the regime (p < 0.01 with n·p or r·(1-p)/p
  in the hundreds or more), too few for a model with a hold-out. T-3960 fills a dense train case and a separate hold-out
  first; this task depends on it.
- Code: `crates/rustsheet-engine/src/functions/statistical/binom.rs`, `bratio/{mod,basym,bfrac,bup,bgrat}.rs`.
  `bratio::dispatch` tells which branch each point takes. Method: docs/research/excel-kernel-playbook.md.

Cells this task owns (T-3774 and T-3959 own the other cells of both cases):
- probes/t3352-binom-negbinom-large-n.case.json (4): Calc!A34, A52, A70, A88
- probes-numeric/t3352-binom-negbinom-sweep.case.json (7): Calc!A132, A147, A156, A171, A184, A190, A208

## Acceptance criteria
- [ ] The Outcome has, for each candidate (including the p' form above and every mixed formation tried), its
      exact / within-4-ulp / within-15-digit counts on T-3960's train case and then once on its hold-out case.
- [ ] Each of the 11 cells above is within half a unit of the 15th significant digit of Excel, or is in statistics.md
      "BINOM.DIST and NEGBINOM.DIST at a large n" as UNVERIFIED with the hypotheses ruled out.
- [ ] The large-n lines A34, A52, A70 and A88 that pass leave eval/baseline/probes.txt (`make eval-baseline`). No new
      known-failure line appears in any eval/baseline file. No gated BETA.DIST, F.DIST, POISSON.DIST,
      BINOM.DIST.RANGE or BINOM.INV cell regresses.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    for the correctly rounded value, on T-3960's split, with the margin stated before scoring;
  - a fresh sweep over the regime, generated and oracle-filled (excel-graph) after the model is frozen (not T-3960's
    cases), with its result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, with every point outside it listed;
  - every fitted constant or cut carries `FITTED(T-3961, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other cdf cells (T-3959) and the pmf (T-3774).
- BINOM.DIST.RANGE and BINOM.INV past the walk (T-3444's follow-ups).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when each of the 11 cells is within half a unit of the 15th significant
digit of Excel, or is an UNVERIFIED statistics.md entry naming the ruled-out hypotheses. Excel is the inaccurate one
here, so this means reproducing Excel's formulation (time-boxed, decision 0056).
