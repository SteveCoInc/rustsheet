---
id: T-2883
title: "Probe name/INDEX/IF forms of a text or boolean cell as data of TRIMMEAN, MODE, paired stats"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Name, INDEX, OFFSET, IF and CHOOSE results holding a text or boolean cell are read like a cell reference in every statistical function that matches `Node::ReferenceKind` by hand.

## Context
T-2344 fixed QUARTILE/PERCENTILE/PERCENTRANK by reading the argument once with `evaluate_picked_reference_arg` (percentile.rs). These still hand-roll `matches!(*x.ungrouped_at(row), Node::ReferenceKind{..})`: trimmean.rs, mode_functions.rs, paired.rs, chisq.rs, z_test.rs, regression.rs, util/aggregate_args.rs, and others (`rg 'Node::ReferenceKind' crates/rustsheet-engine/src/functions`). Probe each with a name/INDEX/IF cell, then share one definition.

## Acceptance criteria
- [x] Oracle-backed probe case per function family; engine matches; statistics.md entries updated; no second definition of the concept.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 14 statistical functions with a lone boolean, text and numeric-text cell given as a plain reference, a defined name, an INDEX result and an IF result (`probes/t2883-stat-name-index-data`, 168 cells, excel-graph). 18 cells failed: TRIMMEAN (`#NUM!` in Excel) and Z.TEST (`#N/A`) read the name/INDEX/IF forms as a typed value. Both now read their data once with `evaluate_picked_reference_arg` (the definition QUARTILE/PERCENTILE use), which also removed their hand-rolled `Node::ReferenceKind` special cases. The other 12 functions already matched. statistics.md updated. Eval delta: probes +168 passing cells and 3 t2775 cells fixed, 0 regressions.
Also fixed (Z.TEST, T-4286's cells): probes/t2775-ztest-single-cell-routes Sheet1!B3, B4 and B13 now pass; T-4286 is closed by this task.
Left, with follow-ups: chisq.rs, regression.rs and aggregate_args.rs still match `Node::ReferenceKind` by hand (CHISQ.TEST, SLOPE and FORECAST already match Excel for these forms in the probe), so "no second definition" holds for TRIMMEAN and Z.TEST only: T-4310 moves the rest. A 3-D reference as TRIMMEAN's or Z.TEST's data is now `#VALUE!` (was `#REF!`), unprobed: T-4309.
