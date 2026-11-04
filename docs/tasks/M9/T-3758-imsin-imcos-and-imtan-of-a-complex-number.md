---
id: T-3758
title: "IMSIN, IMCOS and IMTAN of a complex number reduce a large real part as SIN does"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The complex IMSIN, IMCOS and IMTAN (and any other IM* trig path that still takes libm's sine) print the same last digits as Excel for a large real part, as the real-number paths already do.

## Context
T-3642's grid (probes/im-trig-magnitude-grid, excel-graph) has 74 cells that are text in both but differ in the
last digits: 45 IMSIN, IMCOS and IMTAN cells at |x| = 700 to 720, 16 IMCSCH cells at |y| = 700 and 710 (J13, J32,
J46, J51, J65, J70, J84, J89, J108, J127, J371:J373, J386:J388; `IMCSCH(COMPLEX(0,710))` is -16586.8544023646i,
ours ...3615i), 10 IMSINH/IMCOSH cells at y = 710 (G/H 203, 222, 241, 401, 416) and 3 one-ulp IMSEC/IMCSC/IMSECH
cells at (705,705) (D161, E161, I161). (The grid's other 41 IMCSCH cells at x = 700 to 709, where ours prints 0, are T-2668's and T-2914's): `IMSIN(COMPLEX(710,1E-300))` is
0.0000602887066915739 in Excel, the same as the real `IMSIN("710")` (x87 reduction, `excel_sin_cos`), while ours,
via libm `x.sin()` in `imsin`, prints ...853; `IMTAN(COMPLEX(710,1))` real part ...901 (ours ...949). Code:
crates/rustsheet-engine/src/functions/engineering/complex/trig.rs (`imsin`, `imcos`, `tan_parts`, `imcsch`'s
`y.sin()`/`y.cos()`, `imsinh`, `imcosh`); engineering.md "IM* trig and hyperbolic functions past SIN's argument limit (T-3642)".
probes/im-trig-argument-cut adds 37 such text cells near 2^27, where libm and the x87 reduction differ by more than the
last digit.
The existing last-digit fits (T-2538, T-3609, fit lab `im-hyperbolic`) must not move.

## Acceptance criteria
- [x] The text cells of probes/im-trig-magnitude-grid outside T-3616's y = 709.78 band match, or the remainder is listed with its owner.
- [x] No currently passing IM* cell changes (probes, probes-numeric, fit lab im-* sweeps).
- [x] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` cuts (T-3642, T-3616).

## Outcome
The remaining libm sines in complex/trig.rs now take the x87 reduction:
- IMTAN's `tan_parts` and IMCOT take sin 2x and cos 2x from the new `double_angle_sin_cos`. It reduces 2x with
  `excel_sin_cos` while |x| < 2^27, even where 2x is past 2^27 (exact there, since n has at most 28 bits).
- IMCSCH's y and `hyperbolic_log_form`'s y (IMSINH and IMCOSH past x = 709) take `trig_sin_cos`.
- IMSIN and IMCOS already did this (T-3726).
- No fitted constant moves; this is SIN's own kernel, so there is no fit.

Eval delta: probes known failures 3899 -> 3836 (63 fixed: 41 in im-trig-magnitude-grid, 22 in im-trig-argument-cut),
0 regressions in every suite (probes-numeric's im-* sweeps included). Remaining text cells, with owners:
- the grid's 41 IMCSCH zeros at x = 700 to 709 and argument-cut J16: T-2668, T-2914;
- the grid's D161, E161, I161 (705+705i, 15th digit): new follow-up T-3851;
- T-3616's y = 709.78 band.
Unit test `large_arguments_take_the_x87_reduction`. engineering.md's "not covered" sentence is now the rule.
