---
id: T-3524
title: "Re-validate CUMIPMT and CUMPRINC's GEOMETRIC_MIN_LEAD and LONG_GEOMETRIC_MIN_LEAD on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
CUMIPMT and CUMPRINC's `GEOMETRIC_MIN_LEAD` (0.3) and `LONG_GEOMETRIC_MIN_LEAD` (3.6) (financial/cumulative.rs) are shown to generalize on a fresh hold-out sweep, or are replaced by the accurate form.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- The cuts choose the geometric-sum form over the per-period form by the leading exponent `(nper-end+1)·L`; tagged
  `FITTED(T-3067, 36 cells, ...)` and `FITTED(T-3067, 96 cells, 33..99998 periods, ...)`. Both cut-points were fitted
  inside wide brackets (0.095..0.398, 2.9..4.3).
- Method: build the engine twice (with the selector, and with the accurate form only) and score both on the old
  cells through the fit lab (`docs/architecture/fit-lab.md`, hold-out split, default margin 5 points), then on the
  fresh sweep. Accuracy-suite tables are refused as sweeps (`--holdout-only` measures a frozen fit only).

## Acceptance criteria
- [x] A fresh log-uniform sweep over the selector's full domain (R4.2), new cells in probes-numeric that no earlier fit
      saw, is oracle-filled and committed.
- [x] The Outcome reports, with the selector and with the accurate form (two engine builds, or fit-lab candidates that
      reproduce each bit for bit on the old cells), the fit lab's train and test counts (T-3360's hold-out split and
      margin) on the old cells and the counts on the fresh sweep.
- [x] (long cut; the short cut is kept by decision 0295's recorded deviation, see the Outcome) If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
      accurate form on test and on the fresh sweep, it is replaced by the accurate form (one `FITTED` tag fewer in
      quality/fitted.json via `scripts/check_fitted.py --update`); otherwise it keeps its `FITTED` tag, with the range
      restated to what was measured, and is not applied outside that range (R4.3).
- [x] The Gate A envelope (R2: within 1e-9 relative of 50-digit mpmath or within 15 significant digits of Excel) holds
      on the fresh sweep, with mpmath values recorded in the Outcome or the accuracy tables.
- [x] No gated cell regresses beyond decision 0083's allowance; a replaced selector's lost bit-exact cells are listed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other selectors split from T-3365 (T-3502, T-3503, T-3523, T-3524, T-3525, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).

## Outcome
`LONG_GEOMETRIC_MIN_LEAD` is replaced by the geometric sum (one `FITTED` tag fewer); `GEOMETRIC_MIN_LEAD` keeps its
tag, restated (T-3524), as decision 0295's recorded deviation, with its form choice filed as T-4349.

- **Fresh sweep** (R4.2): probes-numeric/t3524-cumprinc-lead-fresh-sweep, 1,000 cells, oracle-filled on excel-graph
  2026-11-05 (generator `docs/research/t3524/gen_sweep.py`, seed 3524): CUMPRINC and CUMIPMT on the same arguments,
  the leading exponent `(nper-end+1)·ln(1+rate)` log-uniform in [1E-3, 40] (both cuts and both fit brackets inside),
  rates 1E-4..2, both types, 500 rows of at most 32 periods and 500 of 33 to 3,000. No earlier fit saw it.
- **Second fresh sweep** (review round 1: the first draws no rate below 1E-4): probes-numeric/t3524-cumprinc-tiny-rate-sweep,
  1,000 cells, excel-graph 2026-11-05, the same draw at rates log-uniform in [1E-12, 1E-4] (`gen_sweep.py`, seed
  35241), fit `cumulative-tiny`.
- **Fit lab**: `rustsheet-fitlab cumulative` (the 1,238 old cells: T-3067's probes and every probes and gen-m9 case
  with a literal call) and `cumulative-fresh`, new in `kernels/cumulative.py` with a 60-digit mpmath reference.
  Builds, each `rustsheet-eval` from main fffba5e0 with one of `docs/research/t3524/*.patch` in its own
  `CARGO_TARGET_DIR`: `always-geometric` (both cuts 0), `never-geometric` (both infinite), `no-short-cut`, `no-long-cut`.
  Hold-out split T-3360, margin 5 points, reference `correctly rounded`.

| sweep | candidate | train exact / <=4 ulp / <=15 digits | test exact / <=4 ulp / <=15 digits | verdict |
|---|---|---|---|---|
| old (1,238; 880 train / 358 test) | engine before (both cuts) | 393 (44.7%) / 825 / 833 | 146 (40.8%) / 340 / 342 | accept |
| | no-long-cut (= this PR, bit for bit) | 386 (43.9%) / 825 / 833 | 149 (41.6%) / 340 / 342 | accept |
| | no-short-cut | 379 (43.1%) / 816 / 816 | 147 (41.1%) / 341 / 341 | accept |
| | always-geometric | 372 (42.3%) / 816 / 816 | 150 (41.9%) / 341 / 341 | accept |
| | never-geometric | 333 (37.8%) / 757 / 772 | 122 (34.1%) / 311 / 318 | accept |
| | correctly rounded | 252 (28.6%) / 709 / 723 | 106 (29.6%) / 298 / 303 | reference |
| fresh (1,000; 690 / 310) | engine before (both cuts) | 351 (50.9%) / 614 / 631 | 157 (50.6%) / 265 / 275 | accept |
| | no-long-cut (= this PR) | 382 (55.4%) / 640 / 653 | 184 (59.4%) / 282 / 289 | accept |
| | no-short-cut | 349 (50.6%) / 621 / 634 | 165 (53.2%) / 271 / 278 | accept |
| | always-geometric | 380 (55.1%) / 647 / 656 | 192 (61.9%) / 288 / 292 | accept |
| | never-geometric | 314 (45.5%) / 578 / 610 | 137 (44.2%) / 243 / 264 | accept |
| | correctly rounded | 258 (37.4%) / 600 / 632 | 106 (34.2%) / 267 / 282 | reference |
| tiny (1,000; 701 / 299) | engine before (both cuts) | 227 (32.4%) / 374 / 390 | 81 (27.1%) / 143 / 145 | refuse (margin) |
| | no-long-cut (= this PR) | 335 (47.8%) / 590 / 613 | 140 (46.8%) / 245 / 247 | accept |
| | no-short-cut | 238 (34.0%) / 378 / 390 | 98 (32.8%) / 158 / 160 | accept |
| | always-geometric | 346 (49.4%) / 594 / 613 | 157 (52.5%) / 260 / 262 | accept |
| | never-geometric | 162 (23.1%) / 309 / 332 | 61 (20.4%) / 115 / 124 | refuse (below reference) |
| | correctly rounded | 167 (23.8%) / 512 / 560 | 71 (23.7%) / 218 / 232 | reference |

- **Long cut (3.6)**: within the margin, but it does not beat its other form on test (146 vs 149) or on the fresh
  sweep (508 vs 566; on the 500 long-range cells 184 vs 242 to the bit, 20 visibly off against 3), so it is
  replaced (R4.3). At rates 1E-12..1E-4 the cut is further off: 71 of the 500 long-range cells to the bit with it
  against 238 without, and below a lead of 3.6 2 of 404 against 169 (the cut's per-period and closed forms are
  visibly off there: 465 of the sweep's 1,000 cells against 140). By lead (`docs/research/t3524/analyze.py`, fresh long cells, exact before / after): below 0.01
  15 / 34, 0.01..0.095 39 / 53, 0.095..0.3 47 / 56, 0.3..0.398 10 / 9, 0.398..1 45 / 45, 1..2.9 27 / 43, 2.9..4.3 2 /
  3, above 4.3 69 / 69 (unchanged: the geometric sum took them before).
- **Short cut (0.3)**: within the margin, but the geometric sum at every lead is ahead on the old held-out cells
  (150 vs 149 to the bit; visibly 10 won, 11 lost) and on both fresh sweeps over a short range (260 vs 254 and 265 vs
  237 to the bit; visibly 15 won, 9 lost and 33 won, 18 lost). Below a lead of 1E-3 only old cells back it (the
  sweeps start there). So R4.3 replaces it. Review round 3 measured that build on the gated suites: probes 171
  regressed and 158 fixed, gen-m9 3 regressed (b72d952c5f52cb4d A257, d912671baa1364aa A300, e168be7f6d3f84c2 A379)
  and 4 fixed, ironcalc-calc's Excel-authored `CUMPRINC_CUMIPMT.xlsx` G6, H6, G22, H22 regressed, and 21 more cells
  visibly off in 8 gated cases (1,904 to 1,920 in scope; e.g. `CUMIPMT(1E-10,296,197540,13,19,1)`
  -0.00013127066904417006 against Excel's -0.00013127066813467536), with six Excel-pinned unit tests failing. The
  visible cells cancel (21 lost on the gated suites, 21 won on the sweeps), and decision 0083 puts visible parity
  first, so the cut stays as decision 0295's recorded deviation from R4.3, not as a keep-matched branch: T-4349
  (raised to P1) re-decides it with a decider that beats it on a fresh sweep, or asks the owner.
  `FITTED(T-3524, 2114 cells, at most 32 periods; fresh sweeps rate 1E-12..2 and lead 1E-3..40, old probes rate
  1E-300..1E300; ...)`, applied only over a short range. `scripts/check_fitted.py`: cumulative.rs 2 tags to 1.
- **Gate A envelope (R2)** on both fresh sweeps (`docs/research/t3524/gate_a.tsv`, the 60-digit value per cell): 2,000
  of 2,000 inside, all within 1e-9 of the math (worst 5.1e-13 relative); Excel shows no error on either sweep.
- **Eval delta** (gated): probes 12 fixed (fin-cumipmt-period-bound A7, A8; fin-cumprinc-form-boundaries I1, J3, K5,
  K6, K7, L3, L5, L7, L9, L10), 17 regressed, all 1 or 2 ulp and none visibly off (fin-cumipmt-period-bound A3, A4,
  A6, A11, A27, A30; fin-cumprinc-form-boundaries I2, I4, I7, J4, J5, J7, K1, K2, K3, L2, L4), recorded under decision
  0291; other gated suites unchanged; visible parity 1,904 in-scope cells off, unchanged. probes-numeric (not gated)
  gains the 2,000 sweep cells: the first 566 to the bit and 61 visibly off (44 over a short range, all CUMIPMT, worst
  1,024 ulps; 17 over a long one, 16 of them CUMIPMT, worst 256 ulps), the tiny-rate one 475 to the bit and 140
  visibly off (71 short, all CUMIPMT, worst 3,072 ulps; 69 long, 68 CUMIPMT, worst 2,048 ulps), owned by T-4349.
- Unit tests (crates/rustsheet-engine/src/test/test_fn_cumulative_lift.rs): the five that pinned decision 0291's
  long-range cells to the bit (fin-cumipmt-period-bound A3, A11, A27, A30) now check them within 2 ulps of Excel's
  value through `check_within_two_ulps`, which cites 0291; `each_element_is_the_scalar_call` A3 compares the element
  with its scalar call.
- The `quality/fitted.json` update also lowers annuity.rs from 4 to 3: main already has 3 tags there (a stale lock).
- Findings: docs/excel-semantics/financial.md, "The two cuts on a fresh sweep". Follow-up: T-4349.
