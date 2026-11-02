---
id: T-3447
title: "BINOM.DIST and NEGBINOM.DIST at a large n: the 75 probe cells within 5E-12 of Excel but not 15 digits (Loader pmf and basym residue)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Every cell of probes/t3352-binom-negbinom-large-n is within 15 significant digits of Excel. T-3352 fixed the gross loss
(42 to 133 of 208 within 15 digits; all 208 within 5.2E-12 of the math), and this task owns the 75 cells left over.

## Context
- T-3352's Outcome and statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n: basym and Loader, within 1E-11 of the
  math (T-3352)".
- The pmf cells (past the walk) take `binom.rs` `loader_pmf` on `loader::excel_dbinom_exponent`. They are up to 4.4E-12
  from Excel, for example A117 `BINOM.DIST(499952565,1000000000,0.5,FALSE)` and A187
  `NEGBINOM.DIST(100028284,100000000,0.5,FALSE)` at 2.4E-12, while Excel is itself up to 4.1E-12 off the math.
  Hypothesis, not verified: the rounding of n·p into bd0 and the bd0 series' arithmetic at k around 1E8, which T-2586's
  model does not capture.
- The cdf cells are basym in doubles (`bratio/basym.rs`, Excel's arithmetic not identified), bfrac at a large shape, or
  walk tails. They are within 5E-13 of Excel. A48 `BINOM.DIST(600,600000,0.001,TRUE)` was within 15 digits on main
  (statrs) and is not on basym: ours 0.5108555229841736, Excel 0.5108555229841731.
- Cells (probes/t3352-binom-negbinom-large-n, Calc!):
  - pmf (57): A5, A9, A11, A19, A21, A25, A27, A31, A33, A37, A39, A45, A47, A51, A55, A57, A61, A63, A69, A73, A75, A79, A81, A83, A85, A87, A91, A93, A97, A99, A103, A105, A109, A111, A115, A117, A121, A123, A127, A135, A137, A139, A151, A153, A155, A163, A167, A171, A173, A175, A179, A187, A191, A197, A199, A201, A203
  - cdf (18): A6, A8, A12, A28, A34, A48, A52, A70, A88, A126, A128, A130, A138, A140, A142, A146, A174, A206
- T-3444 added probes/t3444-binom-range-inv-large-n (excel-graph): its BINOM.DIST cells (Range!B, Inv!C, Inv!D) and the
  Range!A cells where Excel's BINOM.DIST.RANGE is its own cdf difference (Range!A equals Range!B in Excel) miss only by
  BINOM.DIST's own last digits, so they are this task's. Cells (probes/t3444-binom-range-inv-large-n):
  - Inv!C (71): C1, C2, C3, C4, C6, C7, C8, C9, C10, C12, C13, C14, C18, C19, C20, C22, C24, C25, C26, C28, C30, C32, C34, C36, C37, C38, C40, C44, C46, C48, C49, C50, C51, C52, C54, C56, C58, C60, C64, C66, C68, C70, C72, C74, C76, C78, C80, C82, C84, C86, C88, C90, C92, C94, C96, C98, C100, C102, C104, C106, C108, C109, C112, C114, C116, C117, C118, C120, C122, C124, C126
  - Inv!D (75): D1, D2, D3, D4, D6, D7, D8, D9, D10, D12, D13, D14, D18, D20, D22, D25, D28, D30, D31, D32, D33, D34, D36, D37, D38, D40, D42, D43, D44, D46, D48, D49, D50, D51, D52, D54, D56, D58, D60, D62, D64, D66, D67, D68, D70, D72, D74, D75, D76, D78, D80, D82, D84, D85, D88, D90, D91, D94, D96, D100, D102, D104, D106, D108, D110, D112, D114, D115, D116, D118, D120, D121, D122, D124, D126
  - Range!A (49): A1, A2, A4, A5, A7, A8, A10, A11, A13, A16, A17, A19, A22, A25, A27, A28, A29, A31, A32, A33, A35, A37, A40, A43, A45, A49, A50, A51, A52, A53, A57, A61, A67, A68, A69, A71, A75, A85, A89, A91, A93, A95, A98, A104, A105, A111, A112, A115, A124
  - Range!B (50): B1, B2, B4, B5, B7, B8, B10, B11, B13, B16, B17, B19, B22, B25, B27, B28, B29, B31, B32, B33, B35, B37, B40, B43, B45, B49, B50, B51, B52, B53, B57, B61, B67, B68, B69, B71, B75, B85, B89, B91, B93, B95, B98, B104, B105, B108, B111, B112, B115, B124
- Score against Excel first; the math reference (T-3352's scratch scorer: 50-digit continued fraction and log-gamma)
  only tells which side is noisier. Any fit follows decision 0188 R4.

## Acceptance criteria
- [ ] All 208 cells of probes/t3352-binom-negbinom-large-n are within 15 significant digits of Excel; no gated cell
      regresses.
- [ ] The Gate A floor on probes-numeric/t3352-binom-negbinom-sweep still holds (0 outside both).
- [ ] What was found is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity on cells already within 15 digits (Phase 1b, decision 0188 R5).

## Fit attempts (decision 0056)
### Attempt 1 (2026-11-01): no form; the engine's model stays
Scored on the 104 pmf cells of probes/t3352-binom-negbinom-large-n against Excel. Each candidate changes only bd0, with
the other pieces exact (60-digit mpmath):

| candidate | bit-exact | within 4 ulp | within 15 digits |
|---|---|---|---|
| engine on `main` (M4 series and the `ph` term) | 13 | 41 | 47 |
| exact bd0 (the math) | 7 | 34 | 41 |
| series, `fl(2k fl(v)) - d` (v and ej stored) | 7 | 35 | 42 |
| series, v and ej at 64 bits, `ej - d` | 6 | 33 | 41 |
| series, `d v - r(ej)` (v stored) | 7 | 35 | 43 |
| series, `d v + r(ej)` (v stored) | 6 | 36 | 42 |
| log form, R in doubles (`k LN(fl(k/M)) + (M - k)`) | 6 | 32 | 33 |
| log form, ratio and LN at 64 bits, t stored | 6 | 34 | 37 |
| log form, exact ratio, LN stored | 7 | 35 | 42 |

None beats the engine, so nothing changed in code. The log forms are ruled out: they are up to 9E-8 off at k = 1E9.

Diagnosis (statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n", `docs/research/t3447/pmf_regress.py`):
- Excel's off-mean error is about |d| 2^-53 (d = k - n p): ±2.4 |d| 2^-53 at most on the probe.
- A least-squares fit on 704 large-n cells of probes-numeric/t2513-r3-binom-pmf-n (runs of consecutive p) recovers
  fl(n p) and fl(n q) (coefficients 0.95 and 0.99) and -r(fl(2kv)) (-1.04, -0.88). Those are the engine's terms.
- It leaves an unexplained rms of 0.77 |d| 2^-53, partly smooth along each run (lag-1 autocorrelation 0.56).
- Ruled out as that rest: the roundings of v, k + M, s v and M v, and fl(2kv) on a v held at 64 bits (coefficients
  near 0), and the rounding of d v (a negligible regressor: unstable coefficients and no fall in the residual).
- The fit on the engine's own values returns its model exactly (residual 0.04), which validates the method.

Next attempt: add candidate regressors to `pmf_regress.py` and keep any whose coefficient is near ±1 and whose
residual falls well below 0.77. Untested candidates:
- the series loop's partial sums stored to double, step by step;
- a different association for the first series term;
- the stirlerr or lf pieces at a large n held at another precision. They are not d-sized, but the smooth part of the
  rest suggests a formula difference, not one rounding.

The cdf cells (basym, bfrac, walk tails) were not attempted.

## Outcome
- Fit attempt 1 found no form, so no engine change and no eval delta. The candidates and counts are under "Fit attempts"
  above.
- Recorded in statistics.md ("BINOM.DIST and NEGBINOM.DIST at a large n"):
  - Excel rounds n p to a double before bd0 (VERIFIED by regression, not a hypothesis any more).
  - Excel's off-mean pmf error is the engine's fl(n p), fl(n q) and -r(fl(2kv)) terms plus an unidentified rest of
    rms 0.77 |d| 2^-53.
  - Script: `docs/research/t3447/pmf_regress.py`.
- The acceptance criteria are not met. The cells and the second attempt move to T-3774 (the pattern of T-3552 and
  T-3731).
