---
id: T-4180
title: "Pivot sources: shared caches, named and dynamic ranges"
milestone: M15
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4141]
---

## Goal
Several pivots share a cache; sources given as defined names (including dynamic OFFSET/INDEX ranges) and Tables resolve at refresh.

## Context
SPEC-0032. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- External sources.
