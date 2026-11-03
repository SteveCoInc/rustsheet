---
id: T-3978
title: "Plain range after a table column delete: shrink to C4:C7 and #REF! kind in table-column-range-delete probes"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A plain range after a table column delete matches Excel: `S!C4:D7` shrinks to `S!C4:C7` when one column is deleted, and a range whose both ends are deleted is `S!#REF!`.

## Context
T-3734 left six cells failing (excel-graph probes, 2026-11-01): F!A4 and F!A5 of
probes/table-edits/table-column-range-delete-end, -delete-start and -both-ends-separate
(`make eval-show CASE=<case>`). They fail in two ways:
- -delete-start: Excel gives 150 and `=SUM(S!C4:C7)`; the engine gives `#REF!` and `=SUM(#REF!:C7)`.
- -both-ends-separate: Excel gives `#REF!` and `=SUM(S!#REF!)`; the engine gives `#ERROR!` and `=SUM(S!C4:#REF!)`.
The `#ERROR!` against `#REF!` is its own error-kind bug. Code: `Displacement::range_corners` (stringify.rs).
T-2851 (research) probes more plain-range shapes but does not list these cells.

## Acceptance criteria
- [ ] The six cells above pass and leave `eval/baseline/probes.txt`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whole-row ranges and multi-row deletes (T-2851).
