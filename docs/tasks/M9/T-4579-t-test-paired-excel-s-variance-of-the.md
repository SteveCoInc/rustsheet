---
id: T-4579
title: "T.TEST paired: Excel's variance of the differences (a negative one-pass variance is #DIV/0!, offset pairs keep their digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine's paired T.TEST and TTEST (type 1) give Excel's result on the two Gate A points they miss,
`accuracy/TTEST/s1` A136 (Excel `#DIV/0!`) and A101 (Excel within 1e-15 of the math, ours 1.6e-9 off), with no other
point of the T.TEST and TTEST tables leaving the envelope; the rule is probed before it is implemented.

## Context
- Found by T-4097 (the T and F test tables, excel-graph 2026-11-06). Both points are in
  `eval/accuracy/known-failures.tsv` with this task as owner. Recorded in statistics.md "T.TEST, TTEST, F.TEST and
  FTEST: Gate A tables (T-4097)".
- A136 `TTEST({-497.50530749878,-0.254481178828704},{-515.88030749878,-18.6294811788287},1,1)` is `#DIV/0!` in Excel.
  The double differences are 18.375 and 18.374999999999996, so the two-pass variance is about 1.3E-29: ours is 4.4E-17 and
  the math 2.3E-17. The one-pass variance VAR.S would start from, (Σd² - (Σd)²/n)/(n - 1), is -1.1E-13 there,
  and it is the only paired point of the tables where that form is negative (it is 0 only for exactly constant
  differences, `#DIV/0!` by the evidenced rule). Hypothesis (one cell): Excel's paired
  test is `#DIV/0!` when the one-pass variance of the differences is 0 or below, before VAR.S's switch to the two-pass
  form (statistics.md "VAR/STDEV switch between a one-pass and a two-pass formula"). Every other paired table point
  whose one-pass variance is positive but takes the switch is a number.
- A101 (ten pairs, x near -108519.3, y near 204025.4, spread about 1E-4): Excel 6.268514994599738E-70, the math
  6.268514994599732E-70, ours 6.268514984538223E-70 (1.6e-9 off the math). The double differences x - y round at about
  3E-11 against deviations of about 1E-4, and the tail magnifies that; Excel keeps the exact differences' digits (x87
  differences, or deviations taken per sample before subtracting: to be found).
- Code: `t_test_p` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs (the `TTestType::Paired` arm).

## Acceptance criteria
- [ ] A probe (excel-probe skill, excel-graph) separates the hypotheses for A136's `#DIV/0!` (pairs with a negative,
      zero and tiny positive one-pass variance of the differences) and for A101's digits; the finding is in statistics.md.
      The same probe takes the reversed argument order of two T-4097 rules, both evidenced only with the special sample
      second: a Welch test of one number before a constant sample (`T.TEST({5},{3,3,3,3},2,3)`, against gen-m9
      607e85cfee83a689 A132) and the overflowing `{1E+200,1E+200,-1E+200}` sample first for T.TEST type 2 and F.TEST
      (`T.TEST({1E+200,1E+200,-1E+200},{1,2,4},1,2)`, `F.TEST({1E+200,1E+200,-1E+200},{1,2,4})`). Where Excel's
      answer is an error, `sample_tests.py`'s rules widen to it.
- [ ] TTEST/s1 A136 and A101 inside the envelope, their known-failure lines removed; no other T.TEST or TTEST table
      point or eval cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Types 2 and 3 and F.TEST (inside the envelope on every table point).
