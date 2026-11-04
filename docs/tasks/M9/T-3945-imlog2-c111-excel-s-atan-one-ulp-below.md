---
id: T-3945
title: "IMLOG2 C111: Excel's atan one ulp below the correctly rounded value at 5.15092871182028"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
probes-numeric/t3643-r4-log-modulus C111, `IMLOG2(COMPLEX(0.190581435720431,0.981671389192299))`, prints Excel's
1.98953694302209i (ours ...210), from a stated model of Excel's `atan` rather than a per-point patch.

## Context
Follow-up of T-3795 (docs/excel-semantics/engineering.md "IMLOG10 and IMLOG2 scale IMLN"). IMLOG2's imaginary part is
Excel's ATAN2 kernel `atan(y/x)` times the double above `log2 e`; at C111 Excel's `atan(5.15092871182028)` is
1.3790419226756174, one ulp below the correctly rounded 1.3790419226756176 (the true value is 0.37 ulp below that),
so Excel's CRT `atan` is not correctly rounded there. Already ruled out (T-3795): `π/2 - atan(1/q)` and
`π/2 - atan(x/y)` for |q| > 1 (fit C111, miss IMLOG10 t3643-imlog10-grid I8, S8 and t3643-r4-log-modulus B101, B167),
x87 fpatan with 64-bit constants (6-12 misses). A lead: probe ATAN itself (`=ATAN(q)-1.379041922675617` scaled to
show the last bits) over a sweep of q > 1 to see whether Excel's ATAN shares the miss and on which band.
Code: `excel_atan2` (functions/math_and_trigonometry/mathematical.rs).

## Acceptance criteria
- [x] C111 matches Excel, or a time-boxed negative fit (decision 0056) names what was tried.
- [x] No ATAN, ATAN2, IMARGUMENT, IMLN, IMLOG10, IMLOG2, IMPOWER or IMSQRT cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-3795 cells (fixed).

## Outcome
C111 is not an `atan` miss, so no IM or ATAN kernel changes; C111 now belongs to T-3932.
- New probe probes/t3945-atan-near-midpoint (excel-graph, 2026-11-03, 214 cells, seed 3945). Excel's ATAN is
  correctly rounded at all 202 stored-double points B1:B202. These include C111's quotient of the nearest doubles
  (B1 = ...176) and 200 hard points over q in [0.0625, 1E8) whose true atan lies within 0.15 ulp of a rounding
  midpoint. No CRT-`atan` model is needed: libm `atan` already matches all 202 cells. For comparison, fdlibm `atan`
  matches only 226 of the 240 ATAN cells of probes/asin-atan-sign-source.
- The ulp comes from the input. Excel reads `0.190581435720431` as 0.19058143572043101, one ulp above the nearest
  double. That holds as a literal, through VALUE, and through the complex text parse (C1, C3, C5, C7, and the low
  bits in C8). This is the T-3658 x87 conversion `round53(round64(m x round64(10^-15)))`, now shown at a 15-digit m
  as well. It makes `y/x` 5.150928711820279 (C9), and the correctly rounded atan of that is Excel's
  1.3790419226756174 (B203, B204). `IMLOG2(COMPLEX(A204,A205))` over the stored nearest doubles is also ...209i
  (C10), because COMPLEX writes text that IMLOG2 parses again.
- Recorded in engineering.md ("IMLOG10 and IMLOG2 scale IMLN", replacing the "CRT atan not modelled" note) and in
  numbers.md ("Decimals reach a double through an x87 multiply", the 15-digit point). T-3932 now owns the 9 off
  cells of the new case and t3643-r4-log-modulus C111.
- Eval delta: fixed 0, regressions 0. The new case adds 214 cells: 205 pass and 9 are new-coverage failures, all
  of them the literal/text conversion. No ATAN, ATAN2 or IM cell changed, because no code changed. Fit guards
  (decision 0188 R4) do not apply: there is no fit and no model change, only probe evidence.
