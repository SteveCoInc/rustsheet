---
id: T-4125
title: "Pivot feature catalog v1 generated from the OOXML schema, Office.js and Microsoft docs"
milestone: M16
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
eval/catalog/pivot-features.tsv lists every pivot feature with id, family, OOXML path and default, `affects` (values/layout/text/none), `web` (yes/no/unknown), scope, stage, status and evidence, built so that nothing in the file format is missed.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §4. A script walks the ECMA-376 transitional XSDs (sml pivotTable and shared) and the [MS-XLSX] extension schemas (x14, x15, xpdl) and emits every element and attribute; each gets a row or an explicit `none` row with its reason. Office.js enums (AggregationFunction, ShowAsCalculation, PivotFilterType, Label/Value/DateFilterCondition, PivotLayoutType, SubtotalLocationType) and the docs list in excel-docs.md cross-check it; anything one source has and the others lack is listed as a finding. [MS-OI29500] notes (excel-docs.md) settle defaults that the standard states differently. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Generator script under tools/ (ruff, mypy strict) with the schemas fetched, not vendored, or vendored with licence noted.
- [ ] TSV committed; a test fails when a schema node has no row.
- [ ] Reconciliation findings listed in docs/research/pivot-tables-2026-11-04/catalog-findings.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The report table (task `report-table`).
