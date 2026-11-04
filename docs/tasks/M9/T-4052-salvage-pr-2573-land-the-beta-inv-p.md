---
id: T-4052
title: "Salvage PR 2573: land the BETA.INV p-near-1 probe and sweep, then re-measure"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The BETA.INV p-near-1 evidence from PR #2573 (T-3351) reaches `main`. Its cells are scored against today's engine, and every cell that still fails has an owner.

## Context
Closed as a stale draft on 2026-11-04 (decision 0264, steward skill "Stale drafts" bucket 5). The branch is kept and read-only: copy files from it with `git show <sha>:<path>`. Do not merge or rebase it. Its baselines are hundreds of commits stale, so regenerate them with `make eval-baseline` and never copy them.
- Source: branch `agent/functions-stat/T-3351-beta-inv-p-near-1` at 384e5327f.
- `beta.rs` has changed in at least 5 BETA.INV PRs since (T-3529 #2668, T-3578 #2873, T-3387 #2877, T-3566 #2919, T-3198 #2997). T-3578 already covers the quantile below the smallest double, so rebasing the branch's `upper_quantile_near_one` is not the path. Measure first.
- Evidence: `eval/suites/probes/cases/t3351-beta-inv-p-near-1.{case,oracle}.json` (254 cells) and `eval/suites/probes-numeric/cases/t3351-beta-inv-sweep.{case,oracle}.json` (200 points, seed 3351), both from excel-graph; plus the statistics.md entry "BETA.INV where the quantile rounds to 1 (T-3351)".
- The branch's follow-ups T-3430 (Excel's AS 109 search at tiny shapes) and T-3431 (a tiny alpha with a large beta) never reached `main`.

## Acceptance criteria
- [ ] Both cases land unchanged, and `make eval-baseline` records them with 0 regressions.
- [ ] The Outcome reports, per stratum, how many of the cells the branch fixed already match on current `main`.
- [ ] The statistics.md entry lands, reconciled with the entries T-3578 and T-3566 added.
- [ ] Each still-failing cell is named by an open task (`make exit-owners`), using T-3351 and new ids for what T-3430 and T-3431 covered. The branch's file names are not reused.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting the branch's engine change. A fix, if one is still needed, is a later task written against current `beta.rs`.
