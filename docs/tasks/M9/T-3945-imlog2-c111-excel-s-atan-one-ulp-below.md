---
id: T-3945
title: "IMLOG2 C111: Excel's atan one ulp below the correctly rounded value at 5.15092871182028"
milestone: M9
lane: functions-engineering
status: todo
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
- [ ] C111 matches Excel, or a time-boxed negative fit (decision 0056) names what was tried.
- [ ] No ATAN, ATAN2, IMARGUMENT, IMLN, IMLOG10, IMLOG2, IMPOWER or IMSQRT cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-3795 cells (fixed).
