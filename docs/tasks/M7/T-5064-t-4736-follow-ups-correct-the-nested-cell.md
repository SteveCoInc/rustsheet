---
id: T-5064
title: "T-4736 follow-ups: correct the nested-cell bounds entry and SPEC-0018, and probe PERCENTOF over a nested cell"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The docs describe what T-4736 made the engine do, and PERCENTOF over a nested GROUPBY/PIVOTBY cell has Excel evidence.

## Context
Excel-fidelity review of PR #3859 (T-4736, 2026-11-07), three MINOR findings left for a follow-up:
- docs/excel-semantics/arrays.md, the "Nested GROUPBY and PIVOTBY cells: bounds" DEVIATION entry (about lines
  1510-1514), still says SUM, COUNT and COUNTA flatten a nested cell and charge the walk budget, that the 200-copy SUM is
  `#VALUE!`, and it names the old unit test `flattening_and_handing_over_copies_is_bounded`. Since T-4736, only MAP,
  REDUCE, SCAN and INDEX reads charge that budget. SUM is `#N/A` without reading the cells, and the test is
  `handing_over_copies_is_bounded`. arrays.md around lines 1026-1027 also still speaks of SUM and COUNT in the future
  tense ("land", "once a refill confirms each").
- docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md line 88 still says ours follows the October reads
  through T-2594's `NestedReading::Flatten`, which T-4736 removed.
- PERCENTOF (`percentof_sums`, crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs) also uses `SUM_ARGS`.
  So over a nested cell it changed from flattening to an `#N/A` array element, which its "an error inside an array is
  #NUM!" rule turns into `#NUM!`. No eval case covers it either way. The old flattening was not evidenced either.

## Acceptance criteria
- [ ] arrays.md's bounds entry and the future-tense lines, and SPEC-0018:88, state T-4736's reads. The paragraph at
      arrays.md ~1036 is split before "T-2625 (probes/...)" (design MINOR).
- [ ] A probe (excel-graph) of PERCENTOF over a nested GROUPBY and PIVOTBY cell, an array cell and a reference-body cell,
      recorded in arrays.md. The engine follows it, or the gap is owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The SUM, COUNT and COUNTA reads themselves (T-4736, done).
