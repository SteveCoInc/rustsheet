---
id: T-4152
title: "Eval adapter renders spec-case pivots through the engine"
milestone: M16
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4151, T-4124]
---

## Goal
Spec cases with a `definition` reach the engine as a pivot built from the source cells and rendered by the engine (no baked `rendered` cells), so the `pivots` suite scores the engine's own render.

## Context
crates/rustsheet-eval/src/spec_pivots.rs (T-2084) bakes cells today; keep that path for old cases. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] pivots-suite cases scored through the engine render; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing the baked path.
