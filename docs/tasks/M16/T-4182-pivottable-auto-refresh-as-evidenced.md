---
id: T-4182
title: "PivotTable Auto Refresh as evidenced"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4130, T-4151]
---

## Goal
If the capability matrix shows Auto Refresh is stored in the file and honoured by Excel for the web, pivots marked so refresh after source edits as Excel does; otherwise this task records why not and is dropped.

## Context
excel-docs.md AUTOREFRESH (2025 Insider). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.
T-4130's answer (pivots.md "PivotTable Auto Refresh is not stored or honoured by Excel for the web";
probes/t4130-pivot-auto-refresh): Excel for the web saves no Auto Refresh setting and does not refresh a pivot after a
source edit, with or without `refreshOnLoad`. Only a desktop-saved file could show a flag; without one this task is
dropped with that reason.

## Acceptance criteria
- [ ] Probe-backed behaviour or a dropped task with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
