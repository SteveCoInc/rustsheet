---
id: T-3192
title: "Incomplete beta bpser: T-3170's open items (b0 >= 8 L diff, the 987 cells T-2505's port moved off, 6 shape-of-1 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
campaign: accuracy
depends_on: [T-3170]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The bratio port's bpser items that T-3170 left open are ported where Excel's form is found, or recorded as negative
results with their scores.

## Context
T-3170 ported Excel's w at a caller's shape of 1 (Excel's expm1 and log1p; statistics.md "Incomplete beta at a shape of
1 outside the closed forms") and found that running algdiv x87-stored for b0 >= 8 changes no cell (T-3170 Progress).
Open:
- the bpser criterion's shortfall at b0 >= 8 on probes-numeric/t2464-r2-bpser-small-shapes (engine 2,859 of 2,868
  within 15 digits against the model's 2,861): it is not algdiv's arithmetic, so diff the model's L and the engine's L
  at the cells `docs/research/t2977/score_branches.py` lists (needs the T-2464 harness back from git history, per its
  docstring);
- the 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the
  research model with the bindings venv (decision 0114): which are the port's and which the model's too;
- the 6 cells T-3170's shape-of-1 w moved off: probes/ftest-decomposition B12 (1 ulp, gated; decision 0170),
  t2220-f-dist-sweep-cdf A191, A195, t2220-f-dist-sweep-d2-1 A16 (1 ulp), t2220-f-dist-sweep-rt A195 (4 ulp) and
  t1745-beta-inv-roundtrip G145 (16 ulp, visibly off; 1 ulp of w near 0.95): a near-tie in Excel's log1p (its 27 known 1-ulp misses are of that kind) or another
  stored step.
- Added by gap triage 2026-10-31 (`scripts/exit_owners.py --all-off` on origin/main 4ff408243 listed it with no open
  owner): probes-numeric/t1745-beta-inv-roundtrip G1 `BETA.DIST(2.4674011002723394E-20,0.5,0.5,TRUE)`, which is
  bpser at a = b = 0.5. Excel 9.999999999999999E-11 is the correctly rounded value (mpmath at 50 digits,
  9.99999999999999957E-11), and ours, 9.999999999999984E-11, is 12 ulp below, visibly off. The case's other unowned
  BETA.DIST cells, where Excel is off the math, went to T-3435.
Tools: `docs/research/t3007/cell_diff.py` for full-eval A/B diffs.

## Acceptance criteria
- [x] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with
      its scores. Item 4 (G1) is ported; items 1-3 moved unchanged to T-3886 (see Outcome).
- [x] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.

## Outcome
- Item 4 (t1745-beta-inv-roundtrip G1) was not bpser's arithmetic: BETA.DIST at a = b = 1/2 is an arctangent closed
  form, 2*ATAN(SQRT(x)/SQRT(1-x))/PI in doubles, reflected as (0.5 - w(y)) + 0.5 above x = 0.9 (statistics.md "BETA.DIST
  at a = b = 1/2 is an arctangent closed form, reflected above x = 0.9"). Ported as `incomplete_beta::half_half_cdf`,
  called from `bratio::beta_dist_lower_tail`.
- First ruled out: bpser with an extended LN fixes G1 but moves 13,322 BETA.DIST cells off (full-eval A/B).
- Model selection (decision 0188 R4: a structural form chosen from ranked candidates, so a hold-out and fresh sweep, no
  FITTED tag): tune set the 49 a = b = 1/2 cells on file (frozen choice: reflection at 0.5, 42 of 49); fresh sweep 1
  (probes-numeric/t3192-beta-half-half, 321 x) rejected the 0.5 reflection (248) and placed the switch between 0.888
  and 0.900; frozen at 0.9; fresh sweep 2 (t3192-beta-half-half-2, 341 x) tested it: 287 bit-exact against 228 direct
  and 253 reflected at 0.5. Ported form: 42/49, 281/321, 287/341 bit-exact, every cell within 2 ulp (bpser: 16/49).
  Scripts and table: docs/research/t3192.
- Eval delta: full-eval A/B against main before the new cases: 54 cells better (BETA.DIST, BETADIST and
  t0969-t-beta-link df-1 cells, G1, H1, I1 among them), 0 worse; gated baselines unchanged (all probes-numeric). New
  coverage: 2 probe cases, 1,304 Excel cells (B, C, D columns), all within 2 ulp; no visibly-off cell unowned.
- Items 1-3 (b0 >= 8 L, T-2505's 987 cells, the 6 shape-of-1 cells) were not reached; they moved unchanged to T-3886
  with the open a = b = 1/2 points (x = 0.5 is 0.5000000000000001 in Excel, 0.5 here).
