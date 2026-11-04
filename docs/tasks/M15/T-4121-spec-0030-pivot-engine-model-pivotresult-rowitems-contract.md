---
id: T-4121
title: "SPEC-0030 pivot engine: model, PivotResult, rowItems contract, refresh and grid ownership"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A merged spec that every PV1-PV5 engine task implements: the full pivot definition model (typed for value-affecting attributes, verbatim for the rest), the `PivotResult` (result tree, then Excel's own rowItems/colItems, then cells), when a pivot refreshes (load with refreshOnLoad, API, create; never on edit until Auto Refresh is evidenced), grid ownership of rendered cells and the `#SPILL!` collapse, directive-5 limits, GETPIVOTDATA over the result, export, and the PivotChart data view.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §5 (architecture sketch), §7 (PivotCharts); SPEC-0015 (today's read-only model, which this extends, not replaces); oracle-spike.md (what Excel writes); libreoffice.md §1 and oss-engines.md §1 (LibreOffice and mog pipelines, as hypotheses). One definition per concept: aggregates, variance, the parser/evaluator for calculated fields and the number formatter are reused. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] docs/specs/SPEC-0030-*.md merged, reviewed by reviewer-design.
- [ ] Lists every later M15 engine task it governs and the order they land in.
- [ ] States the limits (items per field, result nodes, output cells, calculated-item recursion) with the DEVIATION entries they need.

## Out of scope
- Implementation.
