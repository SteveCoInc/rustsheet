---
id: T-4282
title: "Probe F.DIST.RT, FDIST and BETA.DIST at a subnormal result with =(..)=0 companions (T-4026 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/statistics.md says, with evidence, whether Excel's F.DIST.RT, FDIST and BETA.DIST cdf flush a
subnormal result to 0 or keep it, as T-4026 found for F.DIST's cdf; the engine matches what the probe shows.

## Context
- T-4026 (statistics.md "F.DIST's cdf flushes a subnormal result to 0") walked `F.DIST(x,m,n,TRUE)` across the
  smallest normal double at (316, 1) and (40, 1), with a `=(F.DIST(..))=0` companion on every cell (decision 0042:
  Graph shows a kept subnormal as 0, only the companion tells them apart). Excel's cdf is a true 0 below 2.2E-308.
  The engine now flushes there (`fisher.rs` `f_dist_of`, `arithmetic::flush_subnormal`).
- The right tail was not walked away from T-4045's L140 prefactor cut, and BETA.DIST was not walked at all (T-4254
  covers BETA.DIST and the t at T-4045's prefactor edge, not a subnormal final result).
- Design as T-4026 did (docs/research/t4026/mkprobe.py): pick shapes whose tail reaches 1E-305 to 1E-311 on a branch
  that is not L140 bup (for the right tail: swap so that w1 = bpser, for example a large df2 with df1 1 or 2 is closed
  form, so use df1 >= 3), solve x for ln(tail) from -700 to -716 by 0.5 with mpmath, two families per function whose
  bpser prefactors differ, and a companion `=(..)=0` per cell. Put the case in eval/suites/probes (tag M9).

## Acceptance criteria
- [ ] A probes case walks F.DIST.RT, FDIST and BETA.DIST(.,TRUE) across 2.2E-308 with companions, excel-graph filled,
      its math sidecar committed.
- [ ] statistics.md records the rule for each function with the case cells as evidence.
- [ ] The engine matches each function's flush or keep (a unit test per function); no cell newly visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4045's L140 prefactor cut (done) and T-4254's BETA.DIST/t edge there.
- F.DIST's density, which keeps a subnormal Y (statistics.md "F density is not the beta density").
