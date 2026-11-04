---
id: T-4124
title: "Score rendered pivot areas and rowItems against the oracle in the eval"
milestone: M15
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4123]
---

## Goal
The eval scores every captured pivot cell (value, then displayed text) and compares the engine's location and rowItems/colItems with Excel's, reporting both per case and in report.md; failures enter the baseline ratchet like any cell.

## Context
Depends on the `pivots` sidecar section. Until the engine renders pivots, these cells are known failures, recorded in eval/baseline through `make eval-baseline` (never hand-edited). The structural comparison localizes layout bugs before cell values. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Pivot-area cells count as scored cells of their suite; a missing render is a failure, not a skip.
- [ ] A structural mismatch is listed per case in report.md (`pivot layout` section).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making the engine render (PV1 tasks).
