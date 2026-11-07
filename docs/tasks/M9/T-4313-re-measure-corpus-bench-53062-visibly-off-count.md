---
id: T-4313
title: "Re-measure corpus-bench/53062 visibly-off count after T-3046 (needs corpus-bench fetched)"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] Fetch corpus-bench, run `make eval-show CASE=` for 53062 and state the visibly-off count; file a follow-up for any cell left.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
