---
id: T-4579
title: "T.TEST paired: identify Excel's paired-test algorithm (its #DIV/0! and its offset-sample values), then implement it"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Identify the algorithm behind Excel's paired T.TEST and TTEST (type 1), then implement it. Two things need explaining:
when it returns `#DIV/0!` on near-constant differences, and its values on offset samples, where it can be far from
the math. Once that is done, `accuracy/TTEST/s1` A136 (Excel `#DIV/0!`) and A101 (Excel within 1E-15 of the math,
ours 1.6E-9 off) should pass, along with the visibly-off cells of probes/t4579-ttest-paired, with no other T.TEST or
TTEST point leaving the envelope.

The first probe (2026-11-07) refuted the original hypothesis, that a negative one-pass variance gives `#DIV/0!`.
No simple form fits the probe either, so what is left is kernel research first (see Context). If the research shows
that no double-precision form reproduces Excel, the Outcome ranks the forms tried and leaves the cells to a
decision.

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
- Excel stays the target (decision 0300) on the probe's cells more than 1E-4 off the math (13 Off numbers, 5 Div
  numbers).
- Probe 1 (T-4579, 2026-11-07): probes/t4579-ttest-paired refutes A136's one-pass hypothesis and shows that Excel's
  paired test is erratic on offset samples: `#DIV/0!` or up to 5.4E11 relative off the math, see statistics.md "Paired T.TEST is
  erratic ...". No simple form fits, so the next step is research into Excel's algorithm, not a guard. Also owns the
  probe's visibly-off cells: probes/t4579-ttest-paired Div! A1, A3, A4, A5, A6, A8, A9, A10, A11, A12, A13, A14, A15,
  A16, A17, A18, A20, A21, A22, A23, A24, A25, A26, A27; Off! A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13,
  A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24.
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/t4579-ttest-paired-fresh (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [x] A probe (excel-probe skill, excel-graph) separates the hypotheses for A136's `#DIV/0!` (pairs with a negative,
      zero and tiny positive one-pass variance of the differences) and for A101's digits; the finding is in statistics.md.
      The same probe takes the reversed argument order of two T-4097 rules, both evidenced only with the special sample
      second: a Welch test of one number before a constant sample (`T.TEST({5},{3,3,3,3},2,3)`, against gen-m9
      607e85cfee83a689 A132) and the overflowing `{1E+200,1E+200,-1E+200}` sample first for T.TEST type 2 and F.TEST
      (`T.TEST({1E+200,1E+200,-1E+200},{1,2,4},1,2)`, `F.TEST({1E+200,1E+200,-1E+200},{1,2,4})`). Where Excel's
      answer is an error, `sample_tests.py`'s rules widen to it.
- [ ] A model of Excel's paired test, found with the excel-kernel playbook (docs/research/excel-kernel-playbook.md)
      and scored on probes/t4579-ttest-paired with an R4 hold-out and a fresh sweep, or an Outcome that ranks the forms
      tried and shows that none fits.
- [ ] If a model is found: TTEST/s1 A136 and A101 inside the envelope, their known-failure lines removed; no other
      T.TEST or TTEST table point or eval cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Types 2 and 3 and F.TEST (inside the envelope on every table point).
