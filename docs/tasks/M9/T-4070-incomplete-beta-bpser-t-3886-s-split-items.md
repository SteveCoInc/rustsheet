---
id: T-4070
title: "Incomplete beta bpser: T-3886's split items, the b0 >= 8 L diff on t2464-r2-bpser-small-shapes and T-2505's 987 moved probes-numeric cells"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The two bpser items of T-3192 that T-3886 split off are ported where Excel's form is found, or recorded as negative
results with their scores in docs/excel-semantics/statistics.md.

## Context
Split from T-3886 (which recorded the a = b = 1/2 and the shape-of-1 items; docs/research/t3886/RESULTS.md). Both items
need tooling rebuilt before any scoring, which is the bulk of the work:
- The bpser criterion's shortfall at b0 >= 8 on probes-numeric/t2464-r2-bpser-small-shapes (engine 2,859 of 2,868
  within 15 digits against the research model's 2,861; docs/research/t2977/score_branches.out, "w_bpser b0>=8"):
  diff the model's L and the engine's L at the cells `docs/research/t2977/score_branches.py` lists. It needs the T-2464
  harness back from git history (`git show f6e944993^:crates/rustsheet-engine/src/functions/statistical/t2464_harness.rs`,
  deleted by T-2977), ported to today's `bratio` module, and the derived data rebuilt per docs/research/t2506/RESULTS.md
  "Data". The harness stays out of the merged tree (T-2977 deleted it): keep it on the branch for scoring, or under a
  test-only `cfg`.
- The 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the research
  model with the bindings venv (decision 0114; `bindings/python/.venv` is not built by session start, build it with
  maturin): which are the port's and which the model's too.
Tools: `docs/research/t3007/cell_diff.py` for full-eval A/B diffs.

## Acceptance criteria
- [x] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with its
      scores.
- [x] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST at a = b = 1/2 and the 6 shape-of-1 cells (T-3886, recorded); the t3568 column-B cells (T-3764).
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.

## Outcome
Both items are negative results, recorded in statistics.md ("Incomplete beta bpser") and in
docs/research/t4070/RESULTS.md. Neither needs a bpser change: what is left is GAMMALN's last digits (T-2122).
- No harness: the engine's values come from `rustsheet-eval show/run` and its GAMMALN from the bindings.
  `docs/research/t2464/excelmath.py` `excel` is moved to today's `rustsheet.Workbook` API.
- b0 >= 8 L diff (`l_diff.py`): engine 1,484/2,842/2,859 and model 1,489/2,840/2,861 of 2,868 (ex/u4/w15). Every one of
  the 145 differing cells has a different GAMMALN(a0). With the engine's GAMMALN, the model equals the engine on all
  2,868, so L and algdiv are the model's already. The 2 visibly-off cells (A9491, A9492) are GAMMALN(0.186), 1 ulp high.
- The 987 cells (`moved_cells.py`): the pre/post reports from 1d3ea444 and 81c3d27e give exactly 987. Of these, 632
  are bit-exact today, 196 are bit-exact in the model only, 141 are off in both and 18 are not modelled. All 196 are
  GAMMALN's: 178 under the model, and 18 under T-3879's split z, with each GAMMALN giving its own side's value.
- Eval delta: none (docs and research scripts only; no engine change).
