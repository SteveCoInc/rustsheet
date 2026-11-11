---
id: T-90005
title: "Scalar functions and IF in legacy array parameters (LINEST and kin) match Excel"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: foundational
depends_on: []
---

## Goal
A scalar function (LOG, ABS) or IF nested in the array parameters of LINEST, LOGEST, TREND, GROWTH, SLOPE, INTERCEPT, CORREL, RSQ, FORECAST, SUMXMY2 and MMULT/TRANSPOSE in a legacy formula evaluates as Excel's, on and off the data rows.

## Context
Owns: probes/legacy-array-parameters-linest (every visibly-off cell).
- 73 visibly-off cells. Row 9 (inside the data rows 8-13): Excel computes over the whole arrays (e.g. SLOPE(LOG(C8:C13),LOG(A8:A13)) 0.99771...) where ours gives #DIV/0! (7 cells, we implicitly intersect to one point) or #VALUE! (2), and 12 numbers differ.
- Row 20 (off the data rows): Excel still computes 22 of them (array parameters take arrays, no implicit intersection); ours #VALUE!. One cell is #N/A in Excel and #VALUE! in ours.
- Row 22: 27 numbers differ. Two cells are #VALUE! in Excel and numbers in ours: `SUM(MMULT(TRANSPOSE(ABS(C8:C13)),ABS(A8:A13)))` (R9, Excel #VALUE!) and `SUMPRODUCT(LINEST(C8:C13,A8:A13,AK8:AK13))` (Z20).
- Run `make eval-show CASE=probes/legacy-array-parameters-linest` for every cell. Which parameters Excel evaluates as arrays in a legacy formula should land in docs/excel-semantics first.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/legacy-array-parameters-linest matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
