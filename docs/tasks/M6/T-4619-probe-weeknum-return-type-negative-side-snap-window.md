---
id: T-4619
title: "Probe WEEKNUM return_type negative-side snap window (-(2^-22+2^-33), -2^-23, -(2^-23+2^-34), -(2^-22+9*2^-36)) with a Monday state set first"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The size of WEEKNUM's negative-side return_type snap window is evidenced, and the engine matches it.

## Context
- T-4470 verified that a negative return_type floors and that `-2^-30` snaps up to 0, but the only negative point inside the window
  is `-2^-30`. The window may be the full 2^-22 + 2^-33 or half of it, as DATE/TIME use for negatives (`near_integer_floor`).
- The engine uses the full tolerance (`near_integer_count`, weeknum.rs). Entry: dates.md "WEEKNUM return_type snap and negative floor".
- Layout as probes/t4470-weeknum-return-type-edges: A3 `=WEEKNUM(100,2)` sets Monday, A2 the edge type, A1 `=WEEKNUM(100,99)` reads.

## Acceptance criteria
- [ ] An excel-graph probe covers `-(2^-22+2^-33)`, `-2^-23`, `-(2^-23+2^-34)` and `-(2^-22+9*2^-36)`.
- [ ] dates.md records the window and drops "unprobed"; the engine matches every cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Positive-side snap (done, T-4454).

## Outcome
- probes/t4619-weeknum-negative-snap-window (12 cells, excel-graph 2026-11-06): the negative return_type snap window is half
  the positive one (2^-23 + 2^-34, as DATE/TIME): `-2^-23` and `-(2^-23+2^-34)` snap to 0 (`#NUM!`); `-(2^-22+2^-33)` and
  `-(2^-22+9*2^-36)` floor to -1 and read the stale state. Ours used the full tolerance and was off on the first point:
  `fn_weeknum` now reads `near_integer_whole`. Unit test added; dates.md updated, "unprobed" dropped.
- Eval delta: new case fully passing, no baseline regressions (baseline gains the new-case line).
