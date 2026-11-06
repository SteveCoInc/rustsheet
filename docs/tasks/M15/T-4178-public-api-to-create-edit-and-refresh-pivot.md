---
id: T-4178
title: "Public API to create, edit and refresh pivot tables, and the Python binding"
milestone: M15
lane: api
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4151]
---

## Goal
The facade creates a pivot from a definition, edits its fields and options, refreshes it, and returns its result; the Python binding mirrors it.

## Context
SPEC-0032 API section; `Txn::add_pivot_table` is doc-hidden test support today. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] API tests and Python binding tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
