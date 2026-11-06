---
id: T-4541
title: "T.INV.2T and TINV are #NUM! in Excel at p 2.2E-308: probe the cut and match it (Gate A known failures left by T-3386)"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] A probes-numeric case with Excel's values over that p range, and a statistics.md entry with the rule.
- [ ] The four points pass Gate A and their known-failure lines are removed; no passing cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tiny-p R3 marks (T-3386, decision 0311).
