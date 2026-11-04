---
id: T-4051
title: "Salvage PR 2529: land the IRR conventional-flow probe and sweep (not the refuted engine stage)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The oracle evidence from PR #2529 (T-3347) that `main` lacks reaches `main`. The engine stage it proposed does not land: T-3406 refuted it.

## Context
Closed as a stale draft on 2026-11-04 (decision 0264, steward skill "Stale drafts" bucket 5). The branch is kept and read-only: copy files from it with `git show <sha>:<path>`. Do not merge or rebase it. Its baselines are hundreds of commits stale, so regenerate them with `make eval-baseline` and never copy them.
- Source: branch `agent/functions-financial/T-3347-irr-num-near-zero` at c39447908.
- T-3406 (#2562) refuted the PR's `irr_conventional` stage (Newton from v = 1 under any cap), and T-3411 (#3063) has since merged a model of IRR's fallback. T-3406's candidate scorer `docs/research/t3406_irr_secant_candidates.py` still reads this probe and sweep from the branch.
- `main` already has `t3347-irr-conventional-holdout.*` (byte-identical, copied by T-3406).
- Missing from `main`: `eval/suites/probes/cases/t3347-irr-conventional-num.{case.json,oracle.json,roots.tsv}` (42 cells), `eval/suites/probes-numeric/cases/t3347-irr-conventional-sweep.{case.json,oracle.json,roots.tsv}` (120 points), `docs/research/t3347_irr_conventional_sweep.py`, `docs/research/t3347_irr_holdout_score.py`.

## Acceptance criteria
- [x] The files listed as missing land unchanged.
- [x] Where the scorer and research scripts point at the branch, they now point at `main`.
- [x] The branch's financial.md entry "IRR on conventional flows near 0 and -10%" is landed only where `main`'s T-3406 entry does not already say it, and nothing from it contradicts T-3406's refutation.
- [x] T-3347 on `main` notes that #2529's stage was refuted (T-3406) and points at T-3411's model. Set its status from current `main`'s eval (the task's criteria), not from the branch.
- [x] `make eval-baseline` records the new cells, with 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- The `irr_conventional` engine stage, and any other engine change.
- T-3406, T-3407 and T-3408 task files from the branch (`main` has its own).

## Outcome
Landed in #3114 (2026-11-04): t3347-irr-conventional-num and t3347-irr-conventional-sweep (with roots.tsv, unchanged from c39447908, plus math sidecars) and the two t3347 research scripts. No script pointed at the branch, so none needed re-pointing. financial.md gains an evidence-only entry "IRR on conventional flows near 0 and -10% (T-3347's training probes)" that records T-3406's refutation of the draft's Newton-from-v=1 stage. T-3347 names the 36 visibly-off cells; 0 regressions.
