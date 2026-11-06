---
id: T-4171
title: "Pivot grouping in refresh and PivotResult, reusing the M6 group rules"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4170, T-4141]
---

## Goal
A refreshed pivot builds its groups from the source as Excel does, and PivotResult lays out grouped fields in every form, reusing the M6 rules (`pivot/number_groups.rs`, `pivot/date_groups.rs`, the importer's `import/pivots/groups.rs`) rather than a second implementation; manual groups and nested date levels follow T-4170's evidence.

## Context
Rescoped 2026-11-06. M6 implemented grouping at import (group labels become cache items) and a compact one-field layout for grouped spec pivots (`pivot/layout.rs`, T-4235/T-4246). What is missing is grouping during a refresh from the source, grouped fields in multi-field and tabular/outline layouts, and manual and multi-level groups. SPEC-0032 decides where the shared group code lives (directive 3). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] T-4170 probes pass; the M6 grouping probes keep passing; corpus group pivots re-render.
- [ ] No duplicated grouping logic (`scripts/check_duplication.py`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto date grouping on field drop (a UI action).
