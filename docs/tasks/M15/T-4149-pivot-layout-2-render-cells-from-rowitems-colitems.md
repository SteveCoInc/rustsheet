---
id: T-4149
title: "Pivot layout 2: render cells from rowItems/colItems"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4147, T-4137]
---

## Goal
The engine writes the pivot's cells from its rowItems/colItems: captions, item labels (compact indentation is a style, the text is the item), values, page field area, `(blank)`, empty-cell and error captions.

## Context
Evidence from `p-captions` and `p-geometry`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Since the plan (2026-11-06): M6 added `crates/rustsheet-engine/src/pivot/layout.rs` (a compact layout of one row field and one data field, used for grouped spec pivots, T-4235), `pivot/number_groups.rs`, `pivot/date_groups.rs` and `rustsheet-xlsx/src/import/pivots/groups.rs`; build on them, do not add a parallel layout or grouping path.

## Acceptance criteria
- [ ] Pivot-area cells of the PV1 probes match the oracle.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number formats of data fields (PV2).
