---
id: T-4122
title: "Case builder v2: pivot definitions one-to-one with OOXML in spec cases"
milestone: M16
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A spec case can describe any pivot the catalog lists, as a definition JSON that maps one-to-one onto pivotTableDefinition/pivotCacheDefinition (layout flags, fields and axes, items with h/sd/s, data fields with subtotal/showDataAs/baseField/baseItem/numFmtId, filters, groups, calculated fields and items, x14 extensions), and spec2xlsx writes a package Excel for the web opens and renders.

## Context
tools/src/rustsheet_tools/pivot.py writes one compact shape today. The spike (oracle-spike.md runs 2-6) showed what Excel for the web needs: a cache consistent with the source (shared items, records, flags), `<items>` on every axis field, and a target area holding no foreign cells; rowItems/colItems may be omitted. ClosedXML's invariants (oss-engines.md §4) are the refusal list. The old `rendered` path stays for existing cases. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] `pivot_tables` entries accept a `definition` object (schema documented in pivot.py and docs/architecture/eval-harness.md); the old keys keep working, existing sidecars unchanged.
- [x] The builder writes the cache, records and items from the source cells and refuses a definition Excel would refuse or crash on (unit tests per invariant).
- [x] Tabular two-row-field, calculated-field and numeric-group cases from the spike build and are opened by excel-graph (`rustsheet-oracle recalc`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Capturing the rendered cells in sidecars (task `capture`).
- Engine-side reading of `definition`.

## Outcome
Builder v2 in tools/src/rustsheet_tools/pivot_definition.py (wired through pivot.py `_defined`/`_link`); tests in tools/tests/test_pivot_definition.py; the Rust case spec accepts an opaque `definition` and the adapter skips such pivots until T-4152. Live excel-graph runs (2026-11-04) rendered a tabular three-data-field pivot with percentOfTotal, a calculated field with a Top-N filter and descending sort, numeric grouping, and two page fields with a hidden item. Runs found two package rules the builder now follows: case-variant text items make Excel call the file corrupt (merged into one item), and sharedItems flags must follow Excel's (booleans and errors count as strings). Both are recorded in docs/excel-semantics/pivots.md (UNVERIFIED until T-4128/T-4134 commit probes). Branch claude/pivot-tables-research-plan-z14cqn, commit "T-4122: case builder v2". Follow-ups: date grouping and calculated-item helpers belong to T-4170/T-4173 (they extend the builder as their probes need).
