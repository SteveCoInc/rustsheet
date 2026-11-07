---
id: T-4160
title: "Fill gen-pivot PV1 to 20,000 scored cells and file the failures"
milestone: M16
lane: research
status: done
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
- [x] Sidecars committed; failures triaged into tasks.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the failures.

## Outcome
Met by T-4132's first batch (#3968): gen-pivot holds 9 oracle-filled cases (excel-graph, every sidecar's
`input_sha256` matching its case), 104 pivots over a covering array of the PV1 dimensions, and **25,941 scored
cells** (22,816 passing, 87.95%), past the 20,000-cell target. Every failing cell has an owning task (decision 0245,
`scripts/new_case_owners.py` passes): T-4807, T-4914 to T-4922 and T-4924 to T-4927, one per cause. The review and CI
of #3968 cover this task's criteria; no further batch was needed.
