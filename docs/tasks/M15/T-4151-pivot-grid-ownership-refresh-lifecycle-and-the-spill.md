---
id: T-4151
title: "Pivot grid ownership, refresh lifecycle and the #SPILL! collapse"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4149, T-4128]
---

## Goal
Rendered cells are owned by their pivot: a refresh clears the old area and writes the new one; a layout that would meet a foreign cell collapses to `#SPILL!` at the anchor as Excel does; pivots refresh on load when `refreshOnLoad` is set and on `Txn::refresh_pivot`, never on source edits; on load without refreshOnLoad the saved cells are kept.

## Context
oracle-spike.md runs 1 and 3; SPEC-0030; formulas reading pivot cells need no new dependency machinery. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Spike probes for re-render and `#SPILL!` pass through the engine.
- [ ] Structural edits (T-2086) still hold.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto Refresh (task `autorefresh`).
