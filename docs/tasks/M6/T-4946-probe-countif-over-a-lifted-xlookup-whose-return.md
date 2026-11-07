---
id: T-4946
title: "Probe COUNTIF over a lifted XLOOKUP whose return range is wider than one column"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for COUNTIF/SUMIF whose range is a lifted XLOOKUP (`XLOOKUP({2;3},…)`) with a return range wider than one column, where each hit is a 1xN row.

## Context
- T-4742 (#3890) makes a lifted XLOOKUP in a per-element caller's range argument a grid of its elements (`Model::xlookup_reference_elements`, `XlookupGrid::Elements`, `RefGrid::of_values` in functions/xlookup.rs). The probes (probes/t4742-countif-array-range-arg) return a single column only. With a multi-column return, the grid's shape may differ from the lookup_value's shape that `criteria_and_grid_static_result` predicts. The excel-fidelity reviewer raised this on #3890 as a MINOR finding.
- Design: `=COUNTIF(XLOOKUP({2;3},A2:A5,B2:C5),">0")`, the same with a 1x2 lookup_value, all-miss with a text if_not_found, and `ROWS`/`COLUMNS` of each result. Fill with excel-graph (excel-probe skill).

## Acceptance criteria
- [ ] A new probes case filled by excel-graph; lookup.md records the shape Excel gives and whether ours matches (follow-up task if not).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The implementation, if ours differs: a separate task.
