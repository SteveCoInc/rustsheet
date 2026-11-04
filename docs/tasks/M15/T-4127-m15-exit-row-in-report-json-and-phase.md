---
id: T-4127
title: "M15 exit row in report.json and Phase 1 exit script"
milestone: M15
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4126]
---

## Goal
report.json `milestone_exit` has an M15 row measuring the M15 exit criteria (docs/milestones.md), and scripts/phase1_exit.py includes M15 in PHASE1_MILESTONES.

## Context
Decision 0274 puts M15 in Phase 1. Per stage: every in-scope catalog row with >= 30 oracle cells; pivots, gen-pivot and fresh pivot-corpus cells not visibly off; remaining rows owned by a task or out of scope. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Row computed in crates/rustsheet-eval/src/milestone_exit.rs with tests.
- [ ] phase1_exit.py and its tests updated (protected path: cites decision 0274).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing other milestones' rows.
