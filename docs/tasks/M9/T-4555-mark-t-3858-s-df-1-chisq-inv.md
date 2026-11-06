---
id: T-4555
title: "Mark T-3858's df 1 CHISQ.INV.RT cells where Excel is more than G off once T-3858 and T-3337 have both landed"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-3337, T-3858]
---

## Goal
Every cell of T-3858's df 1 probes gets an R3 mark in eval/accuracy/math-over-excel.tsv where Excel is more than G =
1e-4 off its committed math value and the engine returns that math. Decision 0267 item 3 rules the math there, and
decision 0313 sets the rule.

## Context
- T-3337 (decision 0313) makes CHISQ.INV.RT and CHIINV at df 1 return the upper-tail quantile of p. It does so where
  the quantile of the double 1 - p is more than G off that quantile. It marks the 45 cells of t3336 and
  t3337-chisq-df1-r3-fresh where Excel is more than G off.
- T-3858 (branch agent/research/T-3858-chisq-df1-num-cutoff, not merged when T-3337 was written) adds
  probes-numeric/t3858-chisq-df1-num-cutoff and -fresh. Their value cells from p 8.3254E-17 to 1.12E-16 are Excel's
  68.7632522116684, the quantile of 1 - 2^-53. The math there runs from about 69.33 down to 68.76 (at 2^-53 Excel is
  the math), so Excel is up to about 8E-3 off. With both branches merged, the engine returns the math wherever the
  quantile of 1 - p is more than G off, so those cells fail against Excel until they are marked.
- Do it the way T-3337 did (its scratch scripts are described in its Outcome): mark only cells whose committed
  `.math.json` value shows Excel more than G off and the engine within 1e-9. Never mark a `#NUM!` cell.

## Acceptance criteria
- [x] Each such t3858 cell is marked, one line each, under the statistics.md "Tiny-p quantiles" entry, whose T-3337
      region names them. `rustsheet-eval accuracy` reports no bad mark.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` cut itself (T-3858).

## Outcome
Folded into T-3337's PR (#3563), since T-3858 (#3535) merged first and the merge put the two rules together. 88
marks, one per value cell above the `#NUM!` edge: t3858-chisq-df1-num-cutoff 54 (CHISQ.INV.RT column A and CHIINV
column C at rows 5 to 11, 15 to 17 and 72 to 88) and -fresh 34 (columns A and B). At each, the committed math shows
Excel 9.4E-4 to 8.2E-3 off and the engine within 1e-9; no `#NUM!` cell is marked. CHIINV math entries were added to
both sidecars (`rustsheet-accuracy math --suite probes-numeric --function CHIINV`, additions only).
`rustsheet-eval accuracy`: 0 failing, 0 table defects.
