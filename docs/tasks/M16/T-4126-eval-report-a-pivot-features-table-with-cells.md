---
id: T-4126
title: "Eval report: a Pivot features table with cells and pass rate per catalog row"
milestone: M16
lane: harness
status: todo
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
- [ ] Table present in report.md; report.json `pivot_features` rows.
- [ ] A catalog row with scope `in` and no cells shows as uncovered.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Gating on it (task `exit-row`).
