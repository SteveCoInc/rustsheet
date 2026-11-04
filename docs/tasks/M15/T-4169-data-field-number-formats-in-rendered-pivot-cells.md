---
id: T-4169
title: "Data field number formats in rendered pivot cells"
milestone: M15
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
Reuse the formatter (directive 3); displayed text is scored by text_expect. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Text probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats.
