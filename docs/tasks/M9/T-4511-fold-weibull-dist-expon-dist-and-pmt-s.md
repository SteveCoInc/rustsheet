---
id: T-4511
title: "Fold WEIBULL.DIST, EXPON.DIST and PMT's 1 - e^x switch into x87_exp::excel_one_minus_exp"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
One definition of Excel's 1 - e^x (minus Kahan's expm1 below |x| = 1, `1 - EXP(x)` from there on): WEIBULL.DIST's,
EXPON.DIST's and PMT/IPMT/PPMT's cdf or `1-Q` call `x87_exp::excel_one_minus_exp` instead of their own copies of the
switch, with no cell changing.

## Context
- T-3764 added `excel_one_minus_exp` (crates/rustsheet-engine/src/x87_exp.rs) for the incomplete beta's a0 = 1 tail
  (statistics.md "Incomplete beta at a shape of 1 outside the closed forms"). The same switch at 1 is written out three
  more times: `weibull.rs` `weibull_cdf`, `exponential.rs` `expon_cdf` (`EXPON_EXPM1_CUT`, FITTED T-2437) and
  `annuity.rs` `discount_complement` (`PAYMENT_EXPM1_TO`, FITTED T-3014). numbers.md "Excel's expm1 is Kahan's
  formula" lists them. T-3764 left them alone to keep its diff to the beta kernel.
- The three are equivalent to the helper on every input: t >= 0 there, so `t < 1` is `|-t| < 1`, and a NaN takes
  `1 - EXP` in both.
- Moving a FITTED tag changes the per-file counts of `scripts/check_fitted.py`: keep one tag on the shared constant that
  cites the fits (or keep the tags where they are on the constants the helper reads), and run the script's `--update`
  as its docstring allows.

## Acceptance criteria
- [ ] The three callers use `excel_one_minus_exp`; no copy of the |x| < 1 switch is left outside `x87_exp.rs`.
- [ ] `make eval` shows no cell changing (compare report.json and each case's `passed` against main).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whether Excel's own expm1 switches at 1 (numbers.md: not established); PPMT's other `excel_expm1` call in
  `cumulative.rs`.
