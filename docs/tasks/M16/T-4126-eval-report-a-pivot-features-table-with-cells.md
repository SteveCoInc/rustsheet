---
id: T-4126
title: "Eval report: a Pivot features table with cells and pass rate per catalog row"
milestone: M16
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4125, T-4124]
---

## Goal
report.md and report.json have one row per pivot catalog id: oracle cells, cells passing, evidence cases and corpus files, so 'implemented' always means measured.

## Context
Cases name the catalog ids they exercise (a `pivot_features` tag list); scored pivot cells count toward each listed id. Mirrors the function catalog's coverage section. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Table present in report.md; report.json `pivot_features` rows.
- [x] A catalog row with scope `in` and no cells shows as uncovered.
- [x] `make check` green; reviewers approve.

## Out of scope
- Gating on it (task `exit-row`).

## Outcome
- `crates/rustsheet-eval/src/pivot_features.rs` reads eval/catalog/pivot-features.tsv and builds report.json's `pivot_features`, with one row per catalog id:
  - the Excel-evidence pivot cells and how many pass;
  - the cases that name the id;
  - `uncovered` when an in-scope row has no cell.
- A spec case names the ids it exercises in a new `pivot_features` list. Its scored sidecar pivot cells (T-4124) count toward each id.
- report.md gains a "Pivot features" section with per-stage totals, the measured rows, the uncovered rows by stage and family, and any ids that cases name but the catalog lacks.
- probes/t4123-pivot-capture and t4128-pivot-spike are tagged with the layout, aggregation, calculated-field, Top-N, grouping, sort and refreshOnLoad ids they evidence. Later probes tag their own.
- Measuring is not gating; gating is the M16 exit row's task.
