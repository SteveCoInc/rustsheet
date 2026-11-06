---
id: T-4121
title: "SPEC-0032 pivot engine: model, PivotResult, rowItems contract, refresh and grid ownership"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A merged spec that every PV1-PV5 engine task implements: the full pivot definition model (typed for value-affecting attributes, verbatim for the rest), the `PivotResult` (result tree, then Excel's own rowItems/colItems, then cells), when a pivot refreshes (load with refreshOnLoad, API, create; never on edit until Auto Refresh is evidenced), grid ownership of rendered cells and the `#SPILL!` collapse, directive-5 limits, GETPIVOTDATA over the result, export, and the PivotChart data view.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §5 (architecture sketch), §7 (PivotCharts); SPEC-0015 (today's read-only model, which this extends, not replaces); oracle-spike.md (what Excel writes); libreoffice.md §1 and oss-engines.md §1 (LibreOffice and mog pipelines, as hypotheses). One definition per concept: aggregates, variance, the parser/evaluator for calculated fields and the number formatter are reused. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Since the plan (2026-11-06): M6 added `crates/rustsheet-engine/src/pivot/layout.rs` (a compact layout of one row field and one data field, used for grouped spec pivots, T-4235), `pivot/number_groups.rs`, `pivot/date_groups.rs` and `rustsheet-xlsx/src/import/pivots/groups.rs`; build on them, do not add a parallel layout or grouping path.

## Acceptance criteria
- [x] docs/specs/SPEC-0032-*.md merged, reviewed by reviewer-design.
- [x] Lists every later M16 engine task it governs and the order they land in.
- [x] States the limits (items per field, result nodes, output cells, calculated-item recursion) with the DEVIATION entries they need.

## Out of scope
- Implementation.

## Outcome
docs/specs/SPEC-0032-pivot-engine.md:
- **Pipeline:** one pivot pipeline, (definition, cache) to `PivotResult`, in fixed stages. Its contract
  is Excel's `rowItems`/`colItems` encoding.
- **Refresh:** on load with `refreshOnLoad`, on the API, on create; never on edit.
- **Grid:** pivot-owned cells, and the `#SPILL!` collapse.
- **Consumers:** GETPIVOTDATA, export and PivotChart data read the result.
- **Limits:** recorded as directive-5 caps.

It names every later M16 engine task and their order, builds on the existing model, `pivot/layout.rs` and
the group modules, and cites the committed probes as the evidence.
