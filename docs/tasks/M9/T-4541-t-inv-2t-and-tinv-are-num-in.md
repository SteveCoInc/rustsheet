---
id: T-4541
title: "T.INV.2T and TINV are #NUM! in Excel at p 2.2E-308: probe the cut and match it (Gate A known failures left by T-3386)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T.INV.2T and TINV return `#NUM!` wherever Excel does at a tiny p, so Gate A's four known failures at p = 2.2E-308 pass
and their lines leave eval/accuracy/known-failures.tsv.

## Context
- Known failures (moved from T-3386, which is done): accuracy/T.INV.2T/s1 A68 `T.INV.2T(2.22507385850721E-308,9)`
  and A78 (df 12), accuracy/TINV/s1 A68 (df 14) and A78 (df 8). Excel `#NUM!`; the engine returns the finite quantile
  (math 3.9E34 at A68). Error mismatches: no R3 mark excuses them (decision 0188 R3).
- Also owned here (T-4102): accuracy/CONFIDENCE.T/s1 A112, A123 and A134, `CONFIDENCE.T(2.22507385850721E-308,sd,n)`
  at n 38, 68 and 79. Excel gives `#NUM!` and the engine returns the finite margin.
- Evidence so far: T.INV at the same p is a number in Excel (accuracy/T.INV/s1 A68, A78; e-T-3425 A68, A78), and
  T.INV.2T is a number at p 1E-300 (t1531 A10, A58, A106) and 1.38E-300 (t3349 A289). Nothing pins the cut between
  2.2E-308 and 1E-300; one guess is that Excel's p/2 below the smallest normal double (p < 4.45E-308) is `#NUM!`.
- Use the `excel-probe` skill: probe T.INV.2T, TINV and CONFIDENCE.T at p from 2.2E-308 to 1E-300 (including 4.45E-308
  on both sides, and subnormal p) at a few df, then implement the evidenced cut.

## Acceptance criteria
- [x] A probes-numeric case with Excel's values over that p range, and a statistics.md entry with the rule.
- [x] The four points pass Gate A and their known-failure lines are removed; no passing cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tiny-p R3 marks (T-3386, decision 0311).

## Outcome
- Probe probes/t4541-t-inv-2t-tiny-p (excel-graph, 2026-11-06; in the gated `probes` suite, not probes-numeric),
  committed with its predictions (hypothesis: p/2 below the smallest normal double is an error) before the fill:
  T.INV.2T at df 1, 2, 9, 30, TINV at df 2, 30 and CONFIDENCE.T at n 2, 3, 31 over 15 p from 2.2E-308 to 1E-300.
  Every cell matches the hypothesis: p up to 4.4501477170144E-308 (just below 2·DBL_MIN) is an error,
  4.45014771701441E-308 (just above) and every larger p a number, at every df. One refinement: at df 1 T.INV.2T is
  `#DIV/0!`, the existing p = 0, df 1 rule, so the flushed half acts as p = 0; CONFIDENCE.T is `#NUM!` at n 2 too.
  T.INV at the same p is a number. A computed subnormal p is 0 in Excel, and a subnormal literal makes excel-graph
  reject the workbook (the probe computes them). No numeric fit: the cut is the structural flush of p/2.
- Engine: `two_tailed_half_underflows` (t_dist.rs, on `flush_subnormal`) shared by T.INV.2T/TINV (`t_inv_of` takes
  such a p as 0) and CONFIDENCE.T (`#NUM!`, checked after its size checks). Unit test
  `two_tailed_p_with_a_subnormal_half_is_zero`. t_dist.rs's unit tests moved unchanged to t_dist/tests.rs to keep the
  file under the 1000-line gate.
- Gate A: the 7 known failures (T.INV.2T/s1 A68, A78; TINV/s1 A68, A78; CONFIDENCE.T/s1 A112, A123, A134) pass and
  their lines are removed; `rustsheet-eval accuracy` 0 failing. The Gate A definitions are unchanged (their math values
  at those points stay finite; the points pass on Excel's error), so no eval/accuracy math sidecar moved.
- Eval delta: 0 regressions in every suite; probes gains the new case (161 of 185 cells pass; 24 last-ulp
  `NumericUlp` misses of finite quantiles, none visibly off; baseline recorded). statistics.md entry updated.
