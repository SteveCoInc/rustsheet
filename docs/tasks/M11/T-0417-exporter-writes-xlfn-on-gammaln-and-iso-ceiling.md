---
id: T-0417
title: "Exporter writes _xlfn. on GAMMALN and ISO.CEILING that Excel-authored files omit"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The exporter writes GAMMALN and ISO.CEILING the way Excel does, or the evidence says it should not change.

## Context
Found by T-0412's review. Excel-authored crates/rustsheet-xlsx/tests/calc_tests/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx stores `GAMMALN(E:E)` and MATH_AND_TRIGONOMETRY/FLOOR_CEILING.xlsx stores `ISO.CEILING(...)` bare, yet `Function::to_xlsx_string` writes `_xlfn.GAMMALN` and `_xlfn.ISO.CEILING`. Excel accepts `_xlfn.` on classic names (SUM, NORMSDIST, CEILING probed), so values are probably unaffected; GAMMALN and ISO.CEILING were not probed. The case builder also writes them prefixed into oracle workbooks.

## Acceptance criteria
- [x] Probe `_xlfn.GAMMALN` and `_xlfn.ISO.CEILING` (excel-probe skill); remove the two arms with tests citing the workbooks, or record why they stay.
- [x] xlsx-storage.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Evidence: Excel writes both names bare. GAMMALN_GAMMALNPRECISE.xlsx (Microsoft Excel 16.0300) stores `GAMMALN(A2)`
  beside `_xlfn.GAMMALN.PRECISE(A2)`, and FLOOR_CEILING.xlsx (Microsoft Excel Online) stores `ISO.CEILING($A175,B$2)`.
  The probe asked for: the case builder read the exporter's arms and wrote both names prefixed into every oracle
  workbook, and none of the 16,517 GAMMALN and 335 ISO.CEILING excel-graph oracle cells is `#NAME?`. So Excel accepts
  the prefixed forms, values never depended on the arms, and no new case was needed.
- Removed the `GammaLn` and `IsoCeiling` arms of `Function::to_xlsx_string`, so the exporter and the case builder
  write the bare names. New test `test_export_gammaln_and_iso_ceiling_have_no_xlfn_prefix` cites both workbooks.
  docs/excel-semantics/xlsx-storage.md records the finding.
- Eval delta: none (fixed 0, new-coverage 0, regressions 0). Refilling an oracle case with the bare form is not
  needed: Excel stores that form itself.
