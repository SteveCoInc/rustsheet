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
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.

## Outcome
- Item 4 (t1745-beta-inv-roundtrip G1) was not bpser's arithmetic: BETA.DIST at a = b = 1/2 is an arctangent closed
  form, 2*ATAN(SQRT(x)/SQRT(1-x))/PI in doubles (statistics.md "BETA.DIST at a = b = 1/2 is an arctangent closed form,
  reflected above x = 0.9"). Ported as `incomplete_beta::half_half_cdf`, the direct form for every x, called from
  `bratio::beta_dist_lower_tail`. G1 is now Excel's 9.999999999999999E-11.
- First ruled out: bpser with an extended LN fixes G1 but moves 13,322 BETA.DIST cells off (full-eval A/B).
- Model selection (decision 0188 R4: a structural form chosen from ranked candidates, so a hold-out and a fresh sweep;
  no `FITTED` tag). Tune set: the 49 a = b = 1/2 cells on file (arctangent family; asin forms lower). Sweep 1
  (probes-numeric/t3192-beta-half-half, 321 x) ranked the forms; sweep 2 (t3192-beta-half-half-2, 341 x) tested a
  reflection above x = 0.9 that sweep 1 had placed.
- Review round 2 (excel-fidelity) dropped that reflection: it is a cut chosen between forms that differ by 1 ulp on
  every cell, below decision 0112's floor (R4.4) and last-digit work (R5). Excel's switch stays recorded in
  statistics.md as a quirk; porting it is T-3886's, with the R4.4 check or an owner ruling and a pinned-shape Gate A
  batch. No fit remains, so quality/fitted.json is main's and the round-1 decision 0256 is withdrawn (its file
  removed; the number stays used).
- R4.1, the fit lab's hold-out split (`holdout()` in docs/research/t3192/score_half.py: the sha256 split of
  tools/src/rustsheet_tools/fitlab/holdout.py over both sweeps' B and C cells; margin 5 points, DEFAULT_MARGIN, stated
  before scoring). Bit-exact / within 4 ulp / within 15 digits:
  - direct (ported): train 522 / 679 / 679 of 679 (76.9%), test 246 / 304 / 304 of 304 (80.9%);
  - correctly rounded: train 397 / 679 / 679 (58.5%), test 176 / 304 / 304 (57.9%);
  - (switch 0.9, not ported: train 583 (85.9%), test 266 (87.5%)).
  Accepted: no drop from train to test, and it beats the correctly rounded value on test.
- R4.2, fresh sweep after the freeze: t3192-beta-half-half-3 (`make_probe.py --sweep 3`, 300 x: log-uniform on
  1E-300..0.1, uniform on 0.1..0.9, 1 - 10^U(-16, -1)), drawn and filled (excel-graph) after round 2 froze the direct
  form: 239 of 300 bit-exact (correctly rounded 212), every cell within 2 ulp. Gate A envelope with the engine's
  values on all three sweeps' math sidecars (dps 80): 1,604 of 1,604 cells inside, every engine value within 2 ulp of
  Excel, visible 0 (`rustsheet-eval run --suite probes-numeric --filter t3192`: BETA.DIST 737 of 962 bit-exact,
  BETADIST 270 of 321). BETA.DIST's Gate A table (s1) draws the shapes and has no a = b = 1/2 point, so no `r4-T-3192`
  batch; `rustsheet-eval accuracy`: 10,004 points, 0 failing.
- Eval delta: `make eval-check` REGRESSIONS 0 in every gated suite. Cell A/B of this head against the same tree with
  the a = b = 1/2 path off (main's bpser), on probes, probes-numeric, gen-m5, gen-m6, gen-m9 and ironcalc-calc
  (`docs/research/t3007/cell_diff.py`): on cases already on file 52 cells better (G1 among them; BETA.DIST, BETADIST
  and t0969-t-beta-link's df-1 cells) and 4 worse by 1 ulp, all ungated (t1745-beta-inv-roundtrip G5, H5, G6, G186,
  bit-exact through bpser and 1E-16 off now; disclosed, not gated, so no decision 0082 record). New coverage: 3 probe
  cases, 1,604 Excel cells, every one within 2 ulp; no visibly-off cell.
- Items 1-3 (b0 >= 8 L, T-2505's 987 cells, the 6 shape-of-1 cells) were not reached; they moved unchanged to T-3886
  with the open a = b = 1/2 points (x = 0.5 is 0.5000000000000001 in Excel, 0.5 here) and Excel's 0.9 switch.
