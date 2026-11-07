---
id: T-4741
title: "LARGE, SMALL and MEDIAN over a mixed XLOOKUP grid: the first element decides between an error and #N/A (probes/t4704 A49:C49, A58:C58)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4704, T-4714]
---

## Goal
LARGE, SMALL and MEDIAN over a mixed XLOOKUP grid that holds an error element and an off-sheet reference return the error of the *first element in order*: the reference first gives `#N/A`, the error first gives the error.

## Context
- Evidence: probes/t4704-mixed-xlookup-grid-readers (excel-graph, 2026-11-07, T-4704): probes 16-21 (Calc rows 46-61 in steps of three; see the case). `LARGE(XLOOKUP({9;2},..,Data!I1:I3,Data!G1:G2),1)` (A49:C49) and `MEDIAN` of the same (A58:C58) are `#N/A`; with `{2;9}` (error first) LARGE, SMALL, MEDIAN and MAX give `#DIV/0!` and ours matches. Ours gives `#DIV/0!` for the reference-first order (`RefGrid::order_statistic_error` picks the first error element, then `#N/A`).
- Fix: the first element decides (the reference element yields `#N/A` before the later error is reached); T-4714 (todo) changes the same function to the first off-sheet reference; do them in order.

## Acceptance criteria
- [x] Calc A49:C49 and A58:C58 of probes/t4704 pass, with the `{2;9}` order still passing. SMALL and MAX in the reference-first order are not probed; add them to the probe in this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other readers of a mixed XLOOKUP grid (T-4704).

## Outcome
`RefGrid::order_statistic_error` returns `#N/A` as soon as the first element is an off-sheet reference, before it looks for an error element; a value or error first element keeps the old rule. Probe extended (refilled on excel-graph, 2026-11-07; the 103 earlier cells unchanged): SMALL over the reference-first grid `{9;2}` is `#N/A` (Calc A88:C88) and MAX `#DIV/0!` (A91:C91), so MAX takes the error in either order while the order statistics stop at the reference. Recorded in lookup.md and SPEC-0014; unit test added. Eval delta: `probes/t4704-mixed-xlookup-grid-readers` 103 -> 109 passing; six known failures (Calc A49:C49, A58:C58) removed from the baseline, the six new cells pass.
