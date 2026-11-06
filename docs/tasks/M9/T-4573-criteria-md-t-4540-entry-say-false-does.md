---
id: T-4573
title: "criteria.md T-4540 entry: say FALSE does not match the number 0, not that it matches nothing"
milestone: M9
lane: functions-criteria
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4540]
---

## Goal
docs/excel-semantics/criteria.md "AVERAGEIF's numeric-like criteria pick the same rows under a resized average_range"
claims only what probes/t4540-averageif-criteria-average-range shows about a FALSE criterion.

## Context
- PR #3573 (T-4540) excel-fidelity review, MINOR (wording only, no engine change): the entry says `FALSE` "matches
  nothing (`#DIV/0!`)". That holds only because A1:A6 (0, -1, 0.5, text "1", TRUE, 1) has no FALSE cell. What
  H7:M7 show is that FALSE does not match the number 0 in A1 (every column is `#DIV/0!`, not 10).

## Acceptance criteria
- [ ] The entry says FALSE does not match the number 0 (A1), citing probes/t4540-averageif-criteria-average-range
      H7:M7, and no longer says FALSE matches nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The T-4540 entry now says FALSE does not match the number 0 (A1), citing H7:M7. Docs only; eval delta none.
