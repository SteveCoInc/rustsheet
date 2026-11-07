---
id: T-4313
title: "Re-measure corpus-bench/53062 visibly-off count after T-3046 (needs corpus-bench fetched)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
corpus-bench/53062's visibly-off count is measured after T-2636 and T-3046 and any remaining cell has an owner task.

## Context
T-3046 could not measure it: `rustsheet-eval fetch --suite corpus-bench` did not complete in the worker environment (it was 334 before T-2636).

## Acceptance criteria
- [x] Fetch corpus-bench, run `make eval-show CASE=` for 53062 and state the visibly-off count; file a follow-up for any cell left.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`rustsheet-eval fetch --suite corpus-bench` completed in this environment (912 files). `rustsheet-eval show
corpus-bench/53062/1_53062_input.xlsx`: status Pass, 642 of 642 cells checked and passed, 0 deviations, so the
visibly-off count is 0 (it was 334 before T-2636). eval/baseline/corpus-bench.txt already lists the case as fully
passing (`C ... 642`). No cell is left, so no follow-up task. No code, eval or baseline change.
