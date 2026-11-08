---
id: T-4837
title: "Discriminating-probe designer: inputs where two IRR models disagree"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-4834]
---

## Goal
Given two IRR models, a tool finds flows and guesses where they disagree in type or past the 15th digit, and writes a probe spec with each model's prediction per cell.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 0, item 3; method of T-3698 (cells chosen because two rules disagree).

## Acceptance criteria
- [ ] `fitlab/irr_discriminate.py`: random draws over the plan's section 3 axes, then a local search on the flows,
      writing a spec case and a predictions file.
- [ ] Unit test: for the base replica against the RATE driver it finds at least 20 disagreeing inputs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Filling probes (T-4838).
