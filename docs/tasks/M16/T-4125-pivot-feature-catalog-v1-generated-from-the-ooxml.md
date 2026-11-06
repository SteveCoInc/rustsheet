---
id: T-4125
title: "Pivot feature catalog v1 generated from the OOXML schema, Office.js and Microsoft docs"
milestone: M16
lane: harness
status: done
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
- [x] Generator script under tools/ (ruff, mypy strict) with the schemas fetched, not vendored, or vendored with licence noted.
- [x] TSV committed; a test fails when a schema node has no row.
- [x] Reconciliation findings listed in docs/research/pivot-tables-2026-11-04/catalog-findings.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- The report table (task `report-table`).

## Outcome
- `tools/src/rustsheet_tools/pivot_catalog.py` (`rustsheet-pivot-catalog --xsd DIR`) walks the ECMA-376 transitional, x14 and x15 schemas from the pivot parts' roots. The schemas are read from a docx4j checkout pinned at f960df59, not vendored (Apache-2.0; the schemas are ECMA's and [MS-XLSX]'s).
- Output:
  - `eval/catalog/pivot-features.tsv`: 1,634 rows (932 in scope, 406 out, 296 round-trip only), each with id, family, OOXML path, default, affects, scope and stage.
  - `eval/catalog/pivot-schema-nodes.txt`: the node list.
- Ids and the hand-filled `web`/`status`/`evidence` columns survive regeneration.
- `tools/tests/test_pivot_catalog.py` fails when a node has no row, a row has no node, a default differs, or a row's classification is not what the rules give. It runs offline.
- Findings: docs/research/pivot-tables-2026-11-04/catalog-findings.md.
  - Office.js `Automatic` and `Unknown` aggregates have no OOXML value.
  - Value and date `Not*` filters have no counterpart in Office.js (T-4164).
  - Auto Refresh and `#SPILL!` are behaviours with no schema node. The report-table task needs `PVB` rows for them.
  - The `xpdl` schema was not walked; it is Data Model only, out of scope.
  - The MS-OI29500 default inversions are listed.
