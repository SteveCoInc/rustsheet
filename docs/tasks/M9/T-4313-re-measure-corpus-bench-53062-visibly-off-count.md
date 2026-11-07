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
Measured on `origin/main` (d137c90) with corpus-bench fetched (`rustsheet-eval fetch --suite corpus-bench` completed, 912 files) and `rustsheet-eval show corpus-bench/53062/1_53062_input.xlsx` (the case id is the file path, so `make eval-show CASE=corpus-bench/53062` finds nothing): status Pass, 642 of 642 oracle-checked cells pass, 0 deviations (source excel-graph). The visibly-off count is therefore 0, down from 334 before T-2636. No cell is left, so no follow-up is filed. Docs only; no eval or baseline change.
