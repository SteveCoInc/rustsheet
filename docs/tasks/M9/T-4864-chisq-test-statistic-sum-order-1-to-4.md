---
id: T-4864
title: "CHISQ.TEST statistic sum order: 1 to 4 ulp off on small 1-D and 2-D ranges (probes/t1689 Calc A6:A8, A10:A11, A17:A18)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
CHISQ.TEST matches Excel to the last bit on the small 1-D and 2-D ranges of probes/t1689-chisq-test-blanks-whole-columns, or the remaining difference is documented.

## Context
Found in T-1689. The semantics (skipped pairs, degrees of freedom) all match; only the last digits differ. Cells Calc!A6:A8, Calc!A10:A11 and Calc!A17:A18 are 1 to 4 ulp off (e.g. A17 expected 0.8172167710680602, got 0.8172167710680605; A6 expected 0.7735040168804609, got 0.773504016880461). Suspect the order of summation of (a-e)^2/e (A17 is the same data as A6-type cells with actual and expected swapped, so the order or the division form matters) or the df 1 erfc kernel. Code: `fn_chisq_test` in `functions/statistical/chisq.rs`.

## Acceptance criteria
- [ ] The seven cells above are bit-exact, or each is a documented `DEVIATION` with evidence.
- [ ] A hold-out set of ranges (fresh probe) agrees (decision 0056 guards).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
