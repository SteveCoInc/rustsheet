---
id: T-4312
title: "Close out the review MINORs of PRs 3278 (T-4249) and 3267 (T-3783)"
milestone: M6
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4249, T-3783]
---

## Goal
The deferred MINOR review findings of #3278 (T-4249, numeric pivot groups) and #3267 (T-3783, Gate A normal-family
tables) are closed: stale task bookkeeping refreshed, web.md wording made exact, and one duplicated overflow check
factored out.

## Context
Both PRs were approved by the full panel with MINOR findings that, per work-loop step 5, did not start a push on the
approved head. They are listed in each PR body's "Agent review" table.
- T-4249 / web.md "Grouped pivot fields": the UNVERIFIED (T-4287) list omits that the engine's whole-data test counts
  `CacheItem::Date` values as numbers (`group_by_numbers` in crates/rustsheet-engine/src/pivot/number_groups.rs), and
  "`start - interval` for the `<start` item" reads as an exact-match rule although any q in [-2, -1) picks `<start`
  (probes/t4249-pivot-range-group-labels P23:P25). T-4253's Outcome still names `relabeled` / `relabeled_items` and
  says "owned by T-4249"; T-4287 should list the fidelity MINORs of #3278 (UNVERIFIED tags for whole-number labels
  at interval 1, a negative start, large intervals, and q between -2 and -1.5).
- T-3783's task file still has a "Landing notes" section from before #3267 was resumed (sidecars stale, round 2 not
  run), and its Outcome counts are pre-merge (16,472 points / 104 tables / missing 153 to 147; after the merge the
  accuracy run gave 17,072 / 107 / 144).
- tools accuracy defs: several "beyond the double max, raise NoValueError" checks (normal.py `_no_overflow`,
  discrete.py, percentof.py, ceiling_floor.py) could share one `no_overflow` helper in `defs/_common.py`.

## Acceptance criteria
- [ ] web.md's T-4249 text names the date-items-count-as-numbers point as UNVERIFIED (T-4287) and phrases the
  `start - interval` case as an example of the truncation rule.
- [ ] T-4253's Outcome matches the merged code (`order`, `items()`); T-4287 lists the #3278 fidelity MINOR points.
- [ ] T-3783's Landing notes are removed (or reduced to one line saying they were resolved) and its Outcome counts
  are refreshed from a current `rustsheet-eval accuracy` run.
- [ ] One `no_overflow` helper in `tools/src/rustsheet_tools/accuracy/defs/_common.py` replaces the duplicated
  checks, with no table output change (`rustsheet-accuracy verify` 0 differences).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New probes for T-4287's points.
