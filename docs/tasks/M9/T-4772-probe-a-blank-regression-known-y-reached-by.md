---
id: T-4772
title: "Probe a blank regression known_y reached by OFFSET, INDIRECT, LET and XLOOKUP (T-4310 follow-up)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence says whether LINEST, LOGEST, TREND and GROWTH treat a blank `known_y` reached through OFFSET, INDIRECT, LET or XLOOKUP like the plain blank reference (waits for x's error), and ours matches it.

## Context
T-4310 (PR #3829) made `read_known_y_and_x` (statistical/regression.rs) ask `Model::is_reference` for a blank y.
probes/t4310-rank-regression-picked-cells A121:A156 shows that a plain reference, a name, INDEX, IF, CHOOSE and
parentheses to a blank cell all give `#DIV/0!` with `1/0` as x. `is_reference` also says TRUE for OFFSET, INDIRECT,
LET-bound references and XLOOKUP, so those forms now wait for x's error too (`#DIV/0!` where they were `#VALUE!`).
No Excel result backs that yet (excel-fidelity review of #3829, MINOR). See docs/excel-semantics/statistics.md, entry
"RANK, LARGE, SMALL and the regression functions: a cell picked by ... (T-4310)".

## Acceptance criteria
- [x] A probe case covers `LINEST(OFFSET(Data!A4,0,0),1/0)`, `LINEST(INDIRECT("Data!A4"),1/0)`,
      `LINEST(LET(y,Data!A4,y),1/0)` and `LINEST(XLOOKUP(..),1/0)` (blank result), the same forms without x, and
      TREND, LOGEST and GROWTH of each, filled from excel-graph.
- [x] Ours matches every cell, or each off cell has an owner task; the statistics.md entry is extended.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/t4772-blank-y-offset-indirect-let-xlookup (32 cells, filled from excel-graph). Excel gives `#DIV/0!` with
`1/0` as known_x and `#VALUE!` alone for all four forms and all four functions, exactly as the plain blank reference;
the engine already matched, so no engine change. statistics.md entry extended. Eval delta: +32 passing probe cells.
