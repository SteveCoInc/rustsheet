---
id: T-4165
title: "Implement pivot label/value/date filters, Top-N and value sorting"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4164, T-4149]
---

## Goal
PivotResult applies the filters and sorts of the catalog's filtering and sorting families as evidenced.

## Context
SPEC-0032; evidence from `p-filters`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; T-4130's capability matrix: every label, value, date and relative date filter, Top-N, legacy AutoShow and value sort the web renders, pivots.md "Pivot sorts and filters as Excel for the web renders them"; label sorts are T-4768's):
- probes/t4130-pivot-label-filters CBt!A4:A6, CBt!B4:B6, CBtI!A4:A6, CBtI!B4:B6, CBw!A6, CBw!B6, CBwU!A6, CBwU!B6, CCo!A4:A5, CCo!B4:B5, CCoU!A4:A5, CCoU!B4:B5, CEq!A4:A5, CEq!B4:B5, CEqL!A4:A5, CEqL!B4:B5, CEw!A5:A6, CEw!B5:B6, CGe!A4:A7, CGe!B4:B7, CGt!A4:A6, CGt!B4:B6, CGtU!A4:A6, CGtU!B4:B6, CLe!A7, CLe!B7, CLt!A6, CLt!B6, CMix!A4:A5, CMix!B4:B5, CNb!A4:A7, CNb!B4:B7, CNbt!A6:A7, CNbt!B6:B7, CNc!A6:A8, CNc!B6:B8, CNe!A7:A8, CNe!B7:B8, CNew!A4:A7, CNew!B4:B7
- probes/t4130-pivot-value-filters TB!A4:A6, TB!B4:B6, TC!A5:A6, TC!B5:B6, TCv!A5:A6, TCv!B5:B6, TP!A5:A6, TP!B5:B6, TS!A5:A6, TS!B5:B6, VBt!A4:A7, VBt!B4:B7, VEq!A4:A5, VEq!B4:B5, VGe!A5:A7, VGe!B5:B7, VGt!A5:A6, VGt!B5:B6, VLe!A4:A7, VLe!B4:B7, VLt!A4:A6, VLt!B4:B6, VM1!A6:A8, VM1!B6:B8, VM1!C6, VM1!C8, VNbt!A6, VNbt!B6, VNe!A6:A8, VNe!B6:B8
- probes/t4130-pivot-date-filters DBt!A4:A8, DBt!B4:B8, DEq!A4:A5, DEq!B4:B5, DNbt!A7:A13, DNbt!B7:B13, DNe!A7:A16, DNe!B7:B16, DNt!A4:A10, DNt!B4:B10, DNte!A4:A11, DNte!B4:B11, DOld!A7, DOld!B7, DOldEq!A8, DOldEq!B8, DWd!A4:A5, DWd!B4:B5, M1!A5:A6, M1!B5:B6, M10!A4:A5, M10!B4:B5, M11!A4:A5, M11!B4:B5, M12!A4:A5, M12!B4:B5, M2!A4:A5, M2!B4, M3!A4:A5, M3!B4:B5, M4!A4:A5, M4!B4:B5, M5!A4:A5, M5!B4:B5, M6!A4:A5, M6!B4:B5, M7!A4:A5, M7!B4:B5, M8!A4:A5, M8!B4:B5, M9!A4:A5, M9!B4:B5, Q1!A7:A8, Q1!B7:B8, Q2!A4:A7, Q2!B4:B7, Q3!A4:A7, Q3!B4:B7, Q4!A4:A7, Q4!B4:B7
- probes/t4130-pivot-relative-date-filters RlM!A4:A5, RlM!B4, RlQ!A4:A5, RlQ!B4, RlW!A4:A5, RlW!B4, RlY!A4:A5, RlY!B4, RnM!A4:A5, RnM!B4:B5, RnQ!A4:A5, RnQ!B4:B5, RnW!A4:A5, RnW!B4:B5, RnY!A4:A5, RnY!B4:B5, RtM!A5, RtM!B5, RtMv!A5, RtMv!B5, RtQ!A5, RtQ!B5, RtW!A5, RtW!B5, RtY!A5, RtY!B5, Rtoday!A5, Rtoday!B5, Rtom!A4:A5, Rtom!B4:B5, Ryest!A4:A5, Ryest!B4, Rytd!A5, Rytd!B5
- probes/t4130-pivot-filter-attrs FBtn!A4:A5, FBtn!B4:B5, FDgi!A4, FDgi!B4, FEo!A4:A5, FEo!B4:B5, FIcon!A4, FIcon!B4, FMeta!A4:A5, FMeta!B4:B5, FMf0!A4:A7, FMf0!B4:B7, FMf1!A4:A7, FMf1!B4:B7, FMp!A4:A5, FMp!B4:B5, FSort!A4:A5, FSort!B4:B5, FTwo!A4:A8, FTwo!B4:B8, FVals!A4:A5, FVals!B4:B5
- probes/t4130-pivot-field-filters AS!A5:A6, AS!B5:B6, ASb!A4:A6, ASb!B4:B6
- probes/t4130-pivot-sort

## Acceptance criteria
- [ ] p-filters probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Slicers and timelines (PV5).
