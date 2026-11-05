---
id: T-4070
title: "Incomplete beta bpser: T-3886's split items, the b0 >= 8 L diff on t2464-r2-bpser-small-shapes and T-2505's 987 moved probes-numeric cells"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with its
      scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST at a = b = 1/2 and the 6 shape-of-1 cells (T-3886, recorded); the t3568 column-B cells (T-3764).
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.
