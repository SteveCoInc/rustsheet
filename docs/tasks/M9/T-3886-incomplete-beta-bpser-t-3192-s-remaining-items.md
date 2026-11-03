---
id: T-3886
title: "Incomplete beta bpser: T-3192's remaining items (b0 >= 8 L diff, T-2505's 987 moved cells, 6 shape-of-1 cells) and a = b = 1/2 at x = 0.5"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3192]
campaign: accuracy
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The bratio port's bpser items T-3192 did not reach are ported where Excel's form is found, or recorded as negative
results with their scores; and BETA.DIST at a = b = 1/2, x = 0.5 is explained.

## Context
T-3192 took its fourth item (t1745-beta-inv-roundtrip G1) and found BETA.DIST at a = b = 1/2 is an arctangent closed
form, not bpser (statistics.md "BETA.DIST at a = b = 1/2 is an arctangent closed form, reflected above x = 0.9";
docs/research/t3192). Its other three items carry over unchanged (read T-3192's Context for the detail):
- the bpser criterion's shortfall at b0 >= 8 on probes-numeric/t2464-r2-bpser-small-shapes (engine 2,859 of 2,868
  within 15 digits against the model's 2,861): diff the model's L and the engine's L at the cells
  `docs/research/t2977/score_branches.py` lists (needs the T-2464 harness back from git history, per its docstring);
- the 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the research
  model with the bindings venv (decision 0114): which are the port's and which the model's too;
- the 6 cells T-3170's shape-of-1 w moved off: probes/ftest-decomposition B12, t2220-f-dist-sweep-cdf A191, A195,
  t2220-f-dist-sweep-d2-1 A16, t2220-f-dist-sweep-rt A195 and t1745-beta-inv-roundtrip G145.
New from T-3192: `BETA.DIST(0.5,0.5,0.5,TRUE)` is 0.5000000000000001 in Excel (m9-beta-dist-a0p5 A9,
m9-legacy-betadist A19, A20, t1745-beta-inv-roundtrip I4, I180) and 0.5 in the arctangent form; the form's other
1-2 ulp misses (40 cells of probes-numeric/t3192-beta-half-half, 54 of t3192-beta-half-half-2) have no model.
Whether T.DIST at df 1 takes the same form (it fixed t0969-t-beta-link's df-1 BETA.DIST cells) is T-2930's.
Tools: `docs/research/t3007/cell_diff.py` for full-eval A/B diffs; `docs/research/t3192/score_half.py`.

## Acceptance criteria
- [ ] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with
      its scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.
