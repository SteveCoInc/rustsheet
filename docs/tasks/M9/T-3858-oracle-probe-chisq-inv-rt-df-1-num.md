---
id: T-3858
title: "Oracle probe: CHISQ.INV.RT df 1 #NUM! cutoff between 8E-17 and 9E-17, with CHISQ.INV(1-p) at the same p"
milestone: M9
lane: research
status: in-progress
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
- [ ] A probes-numeric case `t3858-chisq-df1-num-cutoff.case.json`, filled on excel-graph (excel-probe skill), with `CHISQ.INV.RT(p,1)` and `CHISQ.INV(1-p,1)` at p = 8E-17 to 9E-17 in steps of 1E-18, and at 2^-53 +/- 1 ulp.
- [ ] statistics.md records the cutoff p and whether `CHISQ.INV(1-p,1)` is `#NUM!` there too.
- [ ] T-3337 links the case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3337).
