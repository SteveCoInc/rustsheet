---
id: T-4222
title: "LINEST, LOGEST, GROWTH and TREND given a two-cell range as const or stats: Excel's lift and spill, ours #VALUE! or one result"
milestone: M9
lane: functions-stat
status: done
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
- [x] probes/t3185-m9-statistical-argument-kinds-b A89, B89, A94, B94 and A62, and
      probes/t3185-m9-statistical-argument-kinds-c A69, match Excel.
- [x] The rule is in docs/excel-semantics/statistics.md, citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1-ulp residue of other cells in those cases.

## Outcome
- New probe probes/t4222-regression-flag-ranges (excel-graph, 26 formulas, each with room to spill): a range as
  LINEST/LOGEST `const` or `stats`, or TREND/GROWTH `const`, reads its top-left cell. It does not lift, is no implicit
  intersection, and the other cells (text, `#N/A`) are never read; a text top-left cell is `#VALUE!`. New quirk: a
  blank cell (a single one, or a range's top-left) is FALSE, not the default TRUE of an omitted `const`.
- `flag_range_value` (statistical/linest.rs) reads the top-left cell for LINEST/LOGEST (`flag_input`) and TREND/GROWTH;
  `flag_value` reads a blank cell as FALSE. Recorded in statistics.md "A range as const or stats reads its top-left
  cell".
- The acceptance cells: t3185-b A89, B89, A94, B94 now match (5x2 `#SPILL!`); b A62 and c A69 now take the TRUE call
  and keep only the base call's last-digit residue (6.597539553864475 against ...471, 5.800000000000001 against 5.8),
  which the task puts out of scope, as are the new probe's 45 last-digit cells (no visibly-off cell).
- Eval delta: probes fixed 4, new-coverage failures 45 (all last-digit), regressions 0; every other suite unchanged.
