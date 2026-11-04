---
id: T-3855
title: "Find the arithmetic behind GAMMA.DIST's #NUM! at shapes from 1E56 (density and cdf share it)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GAMMA.DIST's computed 0 is `#NUM!` at the same huge-shape points as Excel's, from a model of Excel's arithmetic,
not a cut.

## Context
- T-3316 found that at shapes from about 1E56 Excel's 0 becomes `#NUM!` at about a third of the probed x/beta,
  in no monotone order (shape 1E70, beta 1: `#NUM!` at x 1E-2, 1E-10, 1E-26, 1E-42, 1E-54, 1E-66, 1E-86, 1E-98, 2E-10,
  3E-10, 1E-5, 2E-5, 2E-20, 1E-100; 0 at 1E-6, 1E-14, 1E-18, 1E-22, 0.1, 5E-10, 2E-9, 2E-11, 2E-100 and more), and
  not monotone in the shape either (x/beta 2E-10: 0 at 1E55, 1E76 and 1.84E76, `#NUM!` from 1E56 to 1.8E76 otherwise).
  The density at x/beta below 1 is `#NUM!` at exactly the cdf's points (12 of 12 pairs). Evidence:
  probes-numeric/t3316-gamma-cdf-y-sweep, t3316-gamma-cdf-huge-shape, t3316-gamma-cdf-shape-sweep,
  fit-audit-t3271-cdf-cut (docs/excel-semantics/statistics.md "GAMMA.DIST at a huge shape"). The density's own
  cells: probes-numeric/t3316-gamma-density-overflow A20 and A24 (`#NUM!`, the engine 0), fit-audit-t3271-cdf-cut A42.
- The engine keeps T-3271's `HUGE_SHAPE_CDF_CUT` (1.85E76) for both (statistical/gamma.rs `huge_shape_num_error`).
- Look for an intermediate whose rounding decides it: an inf * 0 or inf - inf in a product of a huge and a tiny
  factor (an x87 EXP of a huge argument, `x87_exp.rs`), a Loader or Temme form, or a loop that stops on NaN.
  Test candidates against every probed cell before probing more.
- Opus: kernel research with unclear evidence.
- **Attempt 1 (2026-11-04, decision 0056 time-box, about 15 minutes).** The four cases hold 73 distinct huge-shape
  points (shape >= 1E+40, y = x/beta < 1, Excel 0 or `#NUM!`; 34 `#NUM!`); only (shape, y) matters (the same y
  through different x and beta agrees every time). Ruled out as a single separating feature: any one bit (0 to 63)
  of the double t = shape·LN(y), of t - LNGAMMA(shape) or t - LNGAMMA(shape + 1) (exact lgamma rounded once), of
  t·log2(e), LN(y) or shape·LOG10(y), and any threshold on t/LNGAMMA(shape). Patterns worth testing next:
  - At shape 1E+70 the `#NUM!` y are 1E-2, 1E-5, 2E-5, 1E-10, 2E-10, 3E-10, 2E-20, 1E-26, 1E-42, 1E-54, 1E-66,
    1E-86, 1E-98 and 1E-100: the decades from 1E-26 down recur about every 12 (LN(y) steps of about 27.6, so t steps
    of about 2.8E+71), which looks like a remainder of t (or of t·log2(e)) against a fixed modulus. One x87 `fprem`
    pass without looping on C2 leaves a partial remainder that depends on t's bits; f2xm1 of an out-of-range
    operand is undefined. Model the EXP of t (or of t - LNGAMMA) through that sequence (`x87_exp.rs`) and score it
    against the 73 points before probing.
  - At y = 2E-10 every shape from 1E+56 to 1.85E+76 is `#NUM!` except 1E+76 and 1.84E+76, and 1E+40 to 1E+55
    are 0: whatever separates them is mostly a function of y, with an onset in shape.
  - The extraction script is short: parse the GAMMA.DIST/GAMMADIST cells of the four cases with their oracle
    values, keep shape >= 1E+40 and y < 1.

## Acceptance criteria
- [ ] A model that reproduces every probed huge-shape cell (or the remaining misses listed), implemented in place
  of `HUGE_SHAPE_CDF_CUT`; statistics.md updated; `make eval-check` shows no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The shape/x-overflow density rule (T-3316, pinned).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 54 probes-numeric cells that no open task listed: 54 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/fit-audit-t3271-cdf-cut.case.json (22): S!A4, A6-A7, A9-A11, A13-A17, A28-A29, A32, A34-A40, A42
- probes-numeric/t3316-gamma-cdf-huge-shape.case.json (15): S!A1-A7, A10, A13-A14, A16-A20
- probes-numeric/t3316-gamma-cdf-y-sweep.case.json (15): S!A1, A3, A7, A11, A14, A17, A22, A25-A26, A28, A35-A36, A38, A42, A46
- probes-numeric/t3316-gamma-density-overflow.case.json (2): S!A20, A24

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).
