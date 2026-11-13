---
id: T-90014
title: "A what-if data table reading a table evaluates as Excel's (data-table-reads-a-table probe)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The probe's 22 visibly-off cells match Excel (the requesting repository's T-5644 fix waits on it), or their verdict is recorded.

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/data-table-reads-a-table (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
