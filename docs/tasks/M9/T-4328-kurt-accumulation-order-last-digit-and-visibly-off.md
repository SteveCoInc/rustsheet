---
id: T-4328
title: "KURT accumulation order: last-digit and visibly-off cells split from T-1749"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
KURT matches Excel's value to the last digit, and its visibly-off cells are within 15 significant digits.

## Context
Split from T-1749 (DEVSQ), which owned these KURT cells: T-1749 made DEVSQ the two-pass sum `Σ(x - mean)²`
(`variance::squared_deviations`), which closed every DEVSQ cell it owned but no KURT cell.
- KURT (`count_and_average.rs` `fn_kurt`) is already two-pass: s = sqrt(m2/(n-1)) from the two-pass m2, then
  n(n+1)/((n-1)(n-2)(n-3)) * Σ((x - mean)/s)^4 - 3(n-1)²/((n-2)(n-3)). Taking s from Excel's own STDEV.S
  (`excel_variance`, one-pass with its two-pass switch) instead fixed none of the 4 cells tried (T-1749, unscored
  experiment).
- Visibly off (decision 0083): gen-m9/27601819397dd56e Calc!A200 `=KURT(Data!E4:E11)` Excel 3.9999999999979536, ours
  3.9999999999979607; gen-m9/7bd219ec0c0c5e22 Calc!A279 `=KURT(Data!A1:A12)` Excel -0.7628272385808703, ours
  -0.7628272385808685; gen-m9/c2974c061384fdd9 Calc!A55 `=KURT(Data!A6:A6,Data!D1:D11)` Excel 0.734384233823282, ours
  0.7343842338232829.
- Fitting hints carried over from T-1749: where the exact result is 7 (n = 8, one outlier) Excel gives
  7.000000000000002 and we give 7.0 in eight cases (05d44f6dd6cfc567 A185; 06b4e8f5de409d5e A14, 3e98b26cf6713e77 A359,
  4ab2ba98d26c8a5e A356, 68d8ffb69ff739d8 A25, abc2939ebd1e2365 A34, c18dbf21f11093f3 A307, ee2497a5e683c5d9 A350),
  d8266bc15533702a A46 is Excel 7.0 and ours 6.999999999999998, and `KURT({1,..,10})` is Excel -1.2000000000000002
  (a013a45a8fbcab4f A2, a54c738c697617c3 A41). They pin the order of the n(n+1)/((n-1)(n-2)(n-3)) factor, the Σz⁴
  sum and the 3(n-1)²/((n-2)(n-3)) term. Deferred T-1754's three KURT last-digit cells (probes/stats-evidence-floor-
  t1623 Q36, ironcalc-calc MEADIAN_KURT_SKEW_HARMEAN.xlsx F2, probes/stat-argument-modes H13) are likely the same order.

Owned gen-m9 cells (moved from T-1749): 48 known-failure lines in eval/baseline/gen-m9.txt, 3 visibly off.
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A185, A246
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A14
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A139, A322
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A88
- gen-m9/27601819397dd56e.case.json (2): Calc!A200, A108
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A181
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A246
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A74
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A359
- gen-m9/40425341c5034eed.case.json (1): Calc!A313
- gen-m9/4250f880ab640c49.case.json (1): Calc!A45
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A356
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A314
- gen-m9/68d8ffb69ff739d8.case.json (2): Calc!A25, A330
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A279
- gen-m9/7c20351562a2d71d.case.json (3): Calc!A134, A226, A272
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A277
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A139, A261
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A2
- gen-m9/a54c738c697617c3.case.json (1): Calc!A41
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A75
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A140
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A34
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A363
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A123, A353
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A308
- gen-m9/c18dbf21f11093f3.case.json (3): Calc!A77, A215, A307
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A55
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A267
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A24
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A284
- gen-m9/cde551ea787f713a.case.json (1): Calc!A365
- gen-m9/d8266bc15533702a.case.json (1): Calc!A46
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A377
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A350
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A345
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A38, A222

## Acceptance criteria
- [x] Visible parity: the 3 visibly-off cells above are within half a unit of the 15th significant digit of Excel's
      value and their lines leave eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines.
- [x] The other owned cells pass bit for bit, or the ones left are listed in a bit-level follow-up; a structural model
      chosen from ranked candidates carries the decision 0188 R4 guards in the Outcome.
- [x] docs/excel-semantics/statistics.md records KURT's order.
- [x] `make check` green; reviewers approve.

## Out of scope
- DEVSQ (T-1749). SKEW and SKEW.P were in scope only as callers of the shared standardized moment (Outcome).

## Outcome
- SKEW, SKEW.P and KURT share one `standardized_moment(values, divisor, fourth)` (count_and_average.rs), built from the
  `extended_sum` / `extended_product` / `extended_quotient` / `extended_sqrt` kernels: two-pass mean and m2, z =
  (x - mean)/sqrt(m2/divisor), cubes (z·z)·z, fourth powers (z·z)·(z·z), summed in input order (divisor n - 1 for SKEW
  and KURT, n for SKEW.P). KURT's bias factor n(n+1)/((n-1)(n-2)(n-3)) is formed before it scales Σz⁴. The three copied
  loops are gone; the zero or overflowing m2 stays `#DIV/0!` (T-2432).
- Two visible effects: the twice-rounded FSQRT (`SKEW({1E-15,1,2})`, d208bbafc3bc1997 A262 and T-2791's P!A520) and the
  square-and-square fourth power (`KURT({0.1,42,123456789.123,0.1})`, 27601819397dd56e A200).
- R4.1 hold-out (review rounds 1-2): fit lab fits `moments-skew`, `moments-skew-p` and `moments-kurt`
  (tools/src/rustsheet_tools/fitlab/kernels/moments.py), the `rustsheet-fitlab-holdout/v1` split (30%). The margin, the
  default 5 points of train-to-test exact rate, and the cases (every spec case whose text calls SKEW, SKEW.P or KURT; the
  t4974 adversarial probe left out as R4.2 evidence) were stated in the module before scoring. Candidates: the engine
  (this form), main's build `--build main=` (e74590224), the once-rounded binary64 form with chained powers (the engine
  before T-4328, in Python) and the correctly rounded moment (60-digit mpmath, rounded once). Bit-exact train / test:

  | fit | n (test) | engine (this form) | main build | once-rounded, chained | correctly rounded |
  |---|---:|---|---|---|---|
  | moments-skew | 200 (63) | 137/137, 63/63 accept | 135/137, 63/63 accept | 131/137, 54/63 refuse (9.9 pts) | 22/137, 12/63 |
  | moments-skew-p | 178 (54) | 124/124, 54/54 accept | 124/124, 54/54 accept | 116/124, 49/54 accept | 22/124, 8/54 |
  | moments-kurt | 134 (36) | 98/98, 36/36 accept | 71/98, 23/36 refuse (8.6 pts) | 69/98, 21/36 refuse (12.1 pts) | 14/98, 7/36 |

  Development cells: the KURT cells the form was worked out on (gen-m9/27601819397dd56e A200, 7bd219ec0c0c5e22 A279,
  c2974c061384fdd9 A55) are on the train side; SKEW's d208bbafc3bc1997 A262 is on train as well. The cells the fit
  lab cannot parse (nested calls it does not evaluate: 55, 65 and 41) and the ironcalc xlsx cells are outside the sweep.
- R4.2 fresh sweep: Gate A batches eval/accuracy/tables/{SKEW,SKEW.P,KURT}/r4-T-4328 (`rustsheet-accuracy generate
  --stratum log-uniform`, 120 points each, generated and filled by excel-graph after the form was frozen):
  `rustsheet-eval accuracy` puts all 360 points inside the envelope, 360 of 360 bit-exact against Excel, none outside.
  The adversarial probe probes/t4974-moments-x87-adversarial (PR #4002, 204 cells) is extra evidence: 204 of 204.
- Eval: 57 known-failure lines leave the baseline (52 gen-m9, including all 48 owned cells and the 3 visibly off;
  4 probes, T-1754's deferred Q36 and H13 among them; ironcalc MEADIAN_KURT_SKEW_HARMEAN F2), none added. KURT 295/295,
  SKEW 335/335; Gate B 462 → 464 of 506. Unit tests: test_fn_moments.rs (bit-exact Excel values and the `#DIV/0!` edges).
