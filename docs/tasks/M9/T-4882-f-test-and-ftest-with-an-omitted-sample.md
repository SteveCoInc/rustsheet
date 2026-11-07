---
id: T-4882
title: "F.TEST and FTEST with an omitted sample are #VALUE!, not #DIV/0!"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`F.TEST(,{2;3;3;9})`, `F.TEST({2;3;3;9},)` and the same FTEST calls return Excel's `#VALUE!`.

## Context
- statistics.md "F.TEST of a single value (T-2455)" listed the omitted sample as unprobed (ours: the one-value
  `#DIV/0!`). T-4857's kinds probe (probes/t4857-f-kinds, excel-graph 2026-11-07) shows Excel's `#VALUE!` in either
  position, for both names; every other sample kind there already matches.
- Code: `fisher.rs` `fn_f_test` / `f_test_sample` (an omitted argument arrives as an empty node and reads as a
  zero-point sample today).

Cells (probes/t4857-f-kinds): A605:A606, A659:A660

## Acceptance criteria
- [ ] The four cells match Excel; no other t4857-f-kinds or t2455 cell changes.
- [ ] statistics.md's "F.TEST of a single value" entry drops "unprobed" for the omitted sample; a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other paired functions with an omitted argument (statistics.md lists them as unprobed).
