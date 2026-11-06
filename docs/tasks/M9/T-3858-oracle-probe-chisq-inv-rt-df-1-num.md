---
id: T-3858
title: "Oracle probe: CHISQ.INV.RT df 1 #NUM! cutoff between 8E-17 and 9E-17, with CHISQ.INV(1-p) at the same p"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3337 knows where Excel's `CHISQ.INV.RT(p,1)` turns from a value into `#NUM!` and whether the cutoff is in the p check or in the search.

## Context
- T-3336 (probes-numeric/t3336-chisq-df1-right-tail A250-A253): `CHISQ.INV.RT(p,1)` is `#NUM!` at 6E-17, 7E-17 and 8E-17 and a value at 9E-17, though 1 - p is the double 1 - 2^-53 at all four. statistics.md "CHISQ.INV.RT and CHIINV quirks" records it; `CHISQ.INV(1-8E-17,1)` is unprobed.
- Our engine returns 68.7632522116684 at 6E-17 to 8E-17 (TypeMismatch on A250-A252).

## Acceptance criteria
- [x] A probes-numeric case `t3858-chisq-df1-num-cutoff.case.json`, filled on excel-graph (excel-probe skill), with `CHISQ.INV.RT(p,1)` and `CHISQ.INV(1-p,1)` at p = 8E-17 to 9E-17 in steps of 1E-18, and at 2^-53 +/- 1 ulp.
- [x] statistics.md records the cutoff p and whether `CHISQ.INV(1-p,1)` is `#NUM!` there too.
- [x] T-3337 links the case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3337).

## Outcome
- Rule (probes-numeric/t3858-chisq-df1-num-cutoff, excel-graph, 264 cells): `CHISQ.INV.RT(p,1)` and `CHIINV(p,1)`
  are `#NUM!` for p <= 8.3253E-17 and 68.7632522116684 for p >= 8.3254E-17. `CHISQ.INV(1-p,1)` is 68.7632522116684
  at every probed p, `#NUM!` side included (1 - p is 1 - 2^-53 throughout), so the cutoff is in the right-tail
  functions' own p check, not in the search on 1 - p. The edge is about 0.74988 * 2^-53. It is below 3 * 2^-55,
  which is a value, as are 3 * 2^-55 - 2^-106 and 2^-53 +/- 1 ulp. So it is not the double rounding of
  (0.5 - p) + 0.5, and it is not the upper tail at a round x. No closed form found. Recorded in statistics.md
  "CHISQ.INV.RT and CHIINV quirks". The first fill used 16- and 17-digit literals for the ulp points, which Excel
  cuts to 15 digits (numbers.md); the committed case builds them from powers of two.
- Engine: the session's brief said to implement the cutoff once pinned, ahead of T-3337, which leaves out of scope
  only the landing above the edge. chisq.rs `ONE_DEGREE_RIGHT_TAIL_P_MIN` = 8.32535E-17, the midpoint of the
  bracket, is checked before the df 1 branch's `1 - p` quantile. Unit test `chisq_inv_rt_one_degree_num_edge`.
- R4 guards. This is a cut-point inside a probed bracket, tagged like `RIGHT_TAIL_P_MIN`:
  `FITTED(T-3858, 180 cells, df 1, p 5.5E-17..1.12E-16)`; quality/fitted.json 92 -> 93 by `check_fitted.py --update`.
  - Hold-out (R4.1). The split was fixed before scoring: the A and C cells of the case with sha1("Calc!<cell>")
    mod 10 < 3 are held out, margin 5 points. Every cell is 68.7632522116684 or `#NUM!`, so bit-exact, within
    4 ulp and within 15 digits coincide. Chosen cut: train 127/127, hold-out 49/49. The old "1 - p rounds to 1"
    rule: train 43/127, hold-out 17/49. The correctly rounded quantile (no cut): 0/127 and 0/49.
  - Fresh sweep (R4.2): probes-numeric/t3858-chisq-df1-num-cutoff-fresh, generated after the freeze (40
    log-uniform p in [5.6E-17, 1.1E-16], seed 3858, CHISQ.INV.RT and CHIINV): 80/80 bit-exact. The points nearest
    the cut are 8.1554E-17 (`#NUM!`) and 8.5743E-17 (a value).
  - Gate A envelope: every cell of both cases matches Excel's value or error, so all are inside it.
  - Reproducibility floor: the target is a value/error edge, not bits.
- Eval delta: full run with and without the cut, same build, compared case by case on `passed`.
  probes-numeric/t3336-chisq-df1-right-tail goes from 79 to 82 of 291 (A250:A252 are now `#NUM!`). The new
  t3858-chisq-df1-num-cutoff is 264/264 (148 without the cut), and t3858-chisq-df1-num-cutoff-fresh is 80/80. No
  case lost a cell. probes-numeric is ungated, so eval/baseline does not change.
- Follow-ups: none. df 1.5 at the edge is unprobed (df is truncated, so the engine gives it the same edge). T-3337
  still owns the df 1 landing above the edge.
