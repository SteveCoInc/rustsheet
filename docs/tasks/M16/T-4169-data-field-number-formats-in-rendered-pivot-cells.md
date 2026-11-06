---
id: T-4169
title: "Data field number formats in rendered pivot cells"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4149]
---

## Goal
Rendered value cells carry the data field's number format, so their displayed text matches Excel's.

## Context
Reuse the formatter (directive 3); displayed text is scored by text_expect. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Displayed text of pivot cells is not in sidecars yet (T-4123 captures values only): this task adds a `Range.text` read of each pivot's area to the excel-graph fill, beside the `text` cells it reads today, and stores it in the sidecar's `pivots` entries.

## Acceptance criteria
- [ ] Text probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats.
