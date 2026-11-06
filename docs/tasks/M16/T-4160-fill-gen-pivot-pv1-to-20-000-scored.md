---
id: T-4160
title: "Fill gen-pivot PV1 to 20,000 scored cells and file the failures"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4132, T-4152]
---

## Goal
gen-pivot has at least 20,000 oracle-filled scored cells over the PV1 dimensions, and every failing cell has an owning task.

## Context
Task `gen` provides the generator. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Sidecars committed; failures triaged into tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the failures.
