---
id: T-4429
title: "Probe STEYX with only y's sum of squares overflowing at two and three points, then fold the check into PairedMoments"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
STEYX with only y's Σ(y-ȳ)² overflowing matches Excel at two and three points, and the overflow rule lives in one place in `PairedMoments`.

## Context
- T-4047 (probes/t4047-paired-overflow, statistics.md "Paired regression functions") probed y-only overflow at four points: STEYX is `#NUM!`. `fn_steyx` (functions/statistical/correl.rs) now returns `#NUM!` for any n >= 3 with a non-finite `syy`, but only n = 4 has oracle evidence; for n = 2 ours is still `#DIV/0!` (the n < 3 check comes first), also unprobed.
- Reviewers' MINOR notes on PR #3455: the `syy` check sits beside `PairedMoments::overflow`, which already tests `syy`; once n = 2 and 3 are known, fold it into that method so one place owns the rule.

## Acceptance criteria
- [ ] A filled probe case covers STEYX (and CORREL for reference) with only y's sum of squares overflowing at n = 2 and n = 3; the engine matches every cell.
- [ ] The overflow rule for STEYX is in `PairedMoments`, not duplicated in `fn_steyx`; statistics.md records the findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
