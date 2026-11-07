---
id: T-4767
title: "Oracle capability matrix 3: grouping and slicer/timeline rows"
milestone: M16
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4130]
---

## Goal
Every catalog row of the grouping (125) and slicers-timelines (114) families, and the four record-tuple filtering
rows T-4130 left `unknown`, has `web` filled with yes or no, and each cites a probe and sheet in `evidence`, as T-4129
and T-4130 did for the other families.

## Context
- T-4130 was split to stay size M (its Outcome): it filled filtering, sorting and calculated, and answered the
  three questions. These two families were left `unknown`.
- Grouping needs builder support the case builder lacks: date `rangePr` (`groupBy` days to years, `startDate`,
  `endDate`), discrete (manual) groups (`discretePr`, a `databaseField="0"` group field with `base`/`par`), and
  `groupItems` of each item type (`b`, `d`, `e`, `m`, `n`, `s` and their attributes). T-4122 leaves date grouping to
  T-4170; add only what the rows need, in pivot_definition.py and its Rust mirror
  (crates/rustsheet-eval/src/pivot_definition.rs), with golden parts. The numeric `fieldGroup` is already evidenced
  (probes/t4128-pivot-spike G; web.md, M6 grouping).
- Slicers and timelines need new package parts (slicer cache and slicer parts, the drawing's slicer frame, the
  workbook's `x14:slicerCaches`/`x15:timelineCacheRefs` and defined names, the sheet's `x14:slicerList`): a
  spec2xlsx module like pivot_chart.py (T-4130). The question per row is whether the web opens the file, and whether
  a slicer's or timeline's selection filters the pivot.
- The tuple rows (`tpls`, `tpl@fld`, `tpl@hier`, `tpl@item` under groupItems) are OLAP shapes. T-4130 found tuples
  under a cache shared item refused, and four filtering rows (`pivotCacheRecords/r/{e,m,n,s}/tpls/tpl@hier`,
  PV-0699, PV-0720, PV-0740, PV-0761) opening in scratch files only, so they are `unknown` (pivots.md "Capability
  matrix 2"). This task teaches the builder to write tuples under a record value (Python and the Rust mirror) and
  commits a probe for those four rows too.
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] `web` filled for every grouping and slicers-timelines row and PV-0699, PV-0720, PV-0740, PV-0761, each citing
  a probe and sheet.
- [ ] Findings in docs/excel-semantics/pivots.md; owners for every visibly-off cell (decision 0245).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing grouping (T-4171) or slicers (T-4188).
