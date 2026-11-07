---
id: T-4843
title: "IRR driver freeze: fresh sweep, Gate A, financial.md rewrite"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4840, T-4841]
---

## Goal
The new IRR model is frozen and measured on a fresh sweep and Gate A, and financial.md's IRR sections are one rule.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 7.

## Acceptance criteria
- [ ] A fresh sweep (new seed of docs/research/t3411/gen_sweep.py, long-series quota) filled after the freeze.
- [ ] Gate A IRR result recorded; FITTED tags and ranges updated.
- [ ] financial.md's IRR sections merged into one, the old ones marked superseded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
