---
id: T-4222
title: "LINEST, LOGEST, GROWTH and TREND given a two-cell range as const or stats: Excel's lift and spill, ours #VALUE! or one result"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Given a two-cell range (Data!Z5:Z6, holding 1 and 2) as `const` or `stats`, LINEST, LOGEST, GROWTH and TREND
return what Excel returns.

## Context
- Found by T-3185: probes/t3185-m9-statistical-argument-kinds-b and -c (excel-graph, 2026-11-04).
  - `stats` as the range: Excel's LINEST and LOGEST return `#SPILL!` (b A89, A94), so the result is bigger than the
    1-by-2 array ours spills (b B89, B94 hold our second column). Probe the spilled shape with room to spill first;
    the likely reading is a lift over `stats`.
  - `const` as the range: Excel's GROWTH is 6.597539553864471 (b A62) and TREND 5.8 (c A69), the base call's values,
    where ours is `#VALUE!`. LINEST and LOGEST already match for `const` (b A86, A91, to an ulp).
- Code: the LINEST family in crates/rustsheet-engine/src/functions/statistical/.

## Acceptance criteria
- [ ] probes/t3185-m9-statistical-argument-kinds-b A89, B89, A94, B94, A62 and -c A69 match Excel.
- [ ] The rule is in docs/excel-semantics/statistics.md, citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1-ulp residue of other cells in those cases.
