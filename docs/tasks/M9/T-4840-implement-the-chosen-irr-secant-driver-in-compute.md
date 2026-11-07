---
id: T-4840
title: "Implement the chosen IRR secant driver in compute_irr and retire the cuts it subsumes"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4838, T-4839]
---

## Goal
`compute_irr` runs the driver the probes chose, and every FITTED IRR constant it subsumes is gone.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 2/3 result. Absorbs T-3548 and T-4027.

## Acceptance criteria
- [ ] The new driver in functions/financial/financial_util.rs; cuts it subsumes deleted, the rest tagged.
- [ ] Eval: IRR known failures shrink (`make eval-baseline`); any regression under decision 0083's allowance.
- [ ] Hold-out and fresh-sweep guards of decision 0188 R4 recorded in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last digits (T-4842).
