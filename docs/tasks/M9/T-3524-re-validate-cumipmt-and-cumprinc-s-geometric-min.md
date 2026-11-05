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
- [x] If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
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
tag, restated (T-3524), with its form choice filed as T-4349.

- **Fresh sweep** (R4.2): probes-numeric/t3524-cumprinc-lead-fresh-sweep, 1,000 cells, oracle-filled on excel-graph
  2026-11-05 (generator `docs/research/t3524/gen_sweep.py`, seed 3524): CUMPRINC and CUMIPMT on the same arguments,
  the leading exponent `(nper-end+1)·ln(1+rate)` log-uniform in [1E-3, 40] (both cuts and both fit brackets inside),
  rates 1E-4..2, both types, 500 rows of at most 32 periods and 500 of 33 to 3,000. No earlier fit saw it.
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

- **Long cut (3.6)**: within the margin, but it does not beat its other form on test (146 vs 149) or on the fresh
  sweep (508 vs 566; on the 500 long-range cells 184 vs 242 to the bit, 20 visibly off against 3), so it is
  replaced (R4.3). By lead (`docs/research/t3524/analyze.py`, fresh long cells, exact before / after): below 0.01
  15 / 34, 0.01..0.095 39 / 53, 0.095..0.3 47 / 56, 0.3..0.398 10 / 9, 0.398..1 45 / 45, 1..2.9 27 / 43, 2.9..4.3 2 /
  3, above 4.3 69 / 69 (unchanged: the geometric sum took them before).
- **Short cut (0.3)**: within the margin; the geometric sum at every lead is ahead by a cell on the old test side
  (147 vs 146) and by 6 on the fresh sweep (514 vs 508), but removing the cut puts 38 old cells visibly off (tiny-rate
  interests, e.g. gen-m9 b72d952c5f52cb4d A257 33,554,432 ulps, fin-cumipmt-tiny-rate-structure A1, A3, K3,
  fin-cumipmt-gate-edges A8, E2) against 20 it fixes (fresh: 9 against 13), beyond decision 0083's allowance. It
  stays: `FITTED(T-3524, 1614 cells, at most 32 periods, rate 1E-300..1E300; cut a fit in 0.095..0.398)`, the short-range
  cells of both sweeps, applied only over a short range. The leading exponent does not decide the short-range form
  (below 0.095 the old cells favour the other forms, the fresh ones the geometric sum: 46 vs 57): T-4349.
  `scripts/check_fitted.py`: cumulative.rs 2 tags to 1.
- **Gate A envelope (R2)** on the fresh sweep (`docs/research/t3524/gate_a.tsv`, the 60-digit value per cell): 1,000 of
  1,000 inside, all within 1e-9 of the math (worst 1.4e-13 relative); Excel shows no error on the sweep.
- **Eval delta** (gated): probes 12 fixed (fin-cumipmt-period-bound A7, A8; fin-cumprinc-form-boundaries I1, J3, K5,
  K6, K7, L3, L5, L7, L9, L10), 17 regressed, all 1 or 2 ulp and none visibly off (fin-cumipmt-period-bound A3, A4,
  A6, A11, A27, A30; fin-cumprinc-form-boundaries I2, I4, I7, J4, J5, J7, K1, K2, K3, L2, L4), recorded under decision
  0291; other gated suites unchanged; visible parity 1,904 in-scope cells off, unchanged. probes-numeric (not gated)
  gains the 1,000 sweep cells, 566 to the bit and 61 visibly off (44 over a short range, all CUMIPMT, worst 1,024 ulps;
  17 over a long one, 16 of them CUMIPMT, worst 256 ulps), owned by T-4349.
- Findings: docs/excel-semantics/financial.md, "The two cuts on a fresh sweep". Follow-up: T-4349.
