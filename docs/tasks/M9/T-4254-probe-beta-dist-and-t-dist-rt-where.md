---
id: T-4254
title: "Probe BETA.DIST and T.DIST.RT where L140 bup's prefactor crosses DBL_MIN (T-4045's zero rule, evidenced only on F)"
milestone: M9
lane: functions-stat
status: done
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
  meet only around ln(DBL_MIN). The engine applies it to F's tails only (`bratio::f_closed_or_ported_tails`).
  BETA.DIST and the t share the branch but are unprobed (reviewer-excel-fidelity, T-4045 round 1). BETA.DIST's cdf
  also feeds F.INV's search at a huge df1: a 0 there moved `F.INV(1.43795578761177E-262,107702,58)` (accuracy/F.INV/s1
  A146) off the math, while Excel's own F.INV there (0.0239) is far from the math (0.0401). So check whether Excel's
  BETA.DIST zero explains Excel's F.INV.
- Probe: `BETA.DIST(x,a,b,TRUE)` with a in the millions and b < 40 (b0 = b after bratio's swap, x0 > 0.7), and
  T.DIST.RT at a huge df where t's kernel takes L140 bup+bgrat (check `bratio::dispatch` first). Walk x so that
  ln(x0^a0·y0^c/B(c, a0)) crosses -708.396 in steps of about 1, at two shapes per function, at both c = 1/2 and c = 1.
  Put it in eval/suites/probes unless decision 0267's probe freeze allows a probes-numeric case for it. Score it once
  against today's engine (this is the post-freeze fresh check of T-4045's rule).

## Acceptance criteria
- [x] An excel-graph probe case of BETA.DIST and T.DIST.RT cells around the edge, oracle filled.
- [x] Each cell where Excel and ours disagree on 0 against a number has an open owner task (or this task fixes it), and
      statistics.md "Where the exact 0 starts (T-4045)" records the result for BETA.DIST and the t.
- [x] `make check` green; reviewers approve.

## Out of scope
- F (T-4045 probed it); the df1 = 2 closed-form zeros.

## Outcome
- Probes (excel-graph, 2026-11-06), both in eval/suites/probes (decision 0267's freeze keeps them out of probes-numeric):
  - probes/t4254-beta-t-l140-zero-cut (132 cells): ln ret from -698 to -719 by 1; BETA.DIST at two c = 1/2 and two
    c = 1 shapes; T.DIST.RT at df 1E5 and 1E7.
  - probes/t4254-beta-t-zero-wide (180 cells): -400 to -690 by 10, for the families that were already 0 at -698.
- Result, recorded in statistics.md "Where the exact 0 starts (T-4045)":
  - BETA.DIST follows T-4045's DBL_MIN rule at n = 9 and 10 bup terms, at both c = 1/2 and c = 1.
  - At n = 3 bup terms Excel's 0 starts 20 to 30 nats earlier, between ln ret -680 and -690. This is unmodelled.
  - The t never takes L140 (b = 1/2), yet Excel is 0 from a true tail near e^-690.
  - The F.INV search question (A146) stays with T-4437, because no BETA.DIST rule changed here.
- Scored once against today's engine, as the post-freeze fresh check of T-4045's rule:
  - On the rule's own shapes, every Excel number is matched to the last digits, and every Excel 0 is a number in ours
    (11 + 11 cells), because the rule is applied to F only.
  - Every visibly-off cell of both cases is owned by T-4437 (filed, opus), which also asks for the n = 3 and t
    mechanisms (`scripts/new_case_owners.py` passes).
- No engine change. Eval delta: +312 cells, 127 bit-exact, 185 known failures as new coverage (the 0-against-number
  cells and last-digit NumericClose cells), 0 regressions.
- Follow-ups: T-4437.
