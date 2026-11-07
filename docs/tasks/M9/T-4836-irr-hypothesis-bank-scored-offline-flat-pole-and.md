---
id: T-4836
title: "IRR hypothesis bank scored offline: flat, pole and cap handlers (H1-H12)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4834, T-4835]
---

## Goal
A shortlist of at most four complete IRR models, each scored offline on type and bits with hold-out verdicts, and the cells where they disagree.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 2, hypotheses H1-H12 of section 7.

## Acceptance criteria
- [ ] Every hypothesis H1-H12 and the sensible combinations scored by `irr-type`; the table (train/test type and
      bits, cells gained and lost against main) in the Outcome and in financial.md as refutations or leads.
- [ ] The shortlist with each model's prediction on every probe cell, committed under docs/research/irr-2026-11-07/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Oracle probes (T-4838).
