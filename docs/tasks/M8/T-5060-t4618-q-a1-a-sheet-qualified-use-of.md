---
id: T-5060
title: "t4618 Q!A1: a sheet-qualified use of an undefined name, P!N, is #NAME? in Excel and 1 ours"
milestone: M8
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`=P!N`, a sheet-qualified use of a name that sheet P does not define, is `#NAME?` as in Excel.

## Context
- Found by the Phase 1 owners check (2026-11-07): an M8 exit cell with no open M8 owner. Its probe's task, T-4618, is
  done, so this cell lost its owner.
- probes/t4618-order-after-active-backward.case.json Q!A1: formula `P!N`, Excel `#NAME?`, ours 1. The engine likely
  falls back to a workbook-scoped or other sheet's `N`; Excel does not for a qualified reference.
- Split from T-3610 (M9), which also lists this cell for how the eval scores an oracle cell; this task owns the
  M8 engine fix that the M8 exit row needs.
- Code: name resolution of a sheet-qualified name (crates/rustsheet-engine), T-4618 and T-4642's rules
  (docs/excel-semantics names entries).

## Acceptance criteria
- [ ] t4618-order-after-active-backward Q!A1 is `#NAME?`; no M8 cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
