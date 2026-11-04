---
id: T-4254
title: "Probe BETA.DIST and T.DIST.RT where L140 bup's prefactor crosses DBL_MIN (T-4045's zero rule, evidenced only on F)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for BETA.DIST and T.DIST.RT at the edge of T-4045's zero rule. The rule applies to every caller of
bratio's L140 bup+bgrat branch, but only F.DIST.RT and FDIST are probed.

## Context
- T-4045 (PR #3242): Excel's L140 bup+bgrat tail is exactly 0 when bup's prefactor ret = P·x0·y0/c (P the Beta(c, a0)
  density at y0) is below DBL_MIN. Evidence: probes-numeric/t4045-f-dist-rt-zero-cut, three F families whose brackets
  meet only around ln(DBL_MIN). The code (`bratio/bup.rs` `l140_bup_bgrat`) applies it to BETA.DIST's and the t
  functions' calls of that branch too, which no probe covers (reviewer-excel-fidelity, T-4045 round 1).
- Probe: `BETA.DIST(x,a,b,TRUE)` with a in the millions and b < 40 (b0 = b after bratio's swap, x0 > 0.7), and
  T.DIST.RT at a huge df where t's kernel takes L140 bup+bgrat (check `bratio::dispatch` first). Walk x so that
  ln(x0^a0·y0^c/B(c, a0)) crosses -708.396 in steps of about 1, at two shapes per function, at both c = 1/2 and c = 1.
  Put it in eval/suites/probes unless decision 0267's probe freeze allows a probes-numeric case for it. Score it once
  against today's engine (this is the post-freeze fresh check of T-4045's rule).

## Acceptance criteria
- [ ] An excel-graph probe case of BETA.DIST and T.DIST.RT cells around the edge, oracle filled.
- [ ] Each cell where Excel and ours disagree on 0 against a number has an open owner task (or this task fixes it), and
      statistics.md "Where the exact 0 starts (T-4045)" records the result for BETA.DIST and the t.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F (T-4045 probed it); the df1 = 2 closed-form zeros.
