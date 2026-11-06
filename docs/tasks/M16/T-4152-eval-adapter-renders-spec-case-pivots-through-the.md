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

Owner of the pivot probes' visibly-off cells (decision 0245); they pass once this adapter renders them through the engine (SPEC-0032 rollout step 7; moved from T-4149, whose render lands the cells in unit tests only):
probes/t4134-pivot-items, probes/t4137-pivot-captions, probes/t4138-pivot-geometry,
probes/t4135-pivot-aggregates, probes/t4123-pivot-capture, probes/t4128-pivot-spike (pivot areas
scored since T-4124, decision 0317), probes/t4583-pivot-row-labels, probes/t4582-pivot-item-edges, probes/t4589-pivot-total-order, probes/t4614-pivot-total-order-2, probes/t4585-pivot-field-names, probes/t4616-pivot-total-order-3, probes/t4622-pivot-count-subtotals, probes/t4627-pivot-field-name-collisions, probes/t4612-pivot-date-items-and-casing, probes/t4609-pivot-row-labels-2.

## Acceptance criteria
- [ ] pivots-suite cases scored through the engine render; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing the baked path.
