---
id: T-5060
title: "t4618 Q!A1: a sheet-qualified use of an undefined name, P!N, is #NAME? in Excel and 1 ours"
milestone: M8
lane: engine
status: blocked
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

## Blocked because
Blocked because: no engine change is evidenced. The cell's `#NAME?` does not reproduce: docs/excel-semantics/syntax.md
"Which place a loaded use took" (T-4642) records two refills of the same input bytes (sha256 f800e58a…,
probes/t4642-backward-refill and -refill-b) and five one-variable variants (t4642-qualified-after-active,
-fourth-tab-before, -fourth-tab-after, -qualified-entered, -no-bare-use), all excel-graph 2026-11-07, reading 1 at the
qualified use, which is what the engine returns. The task's premise, that Excel never falls back for a qualified use,
is contradicted by that evidence, and making Q!A1 `#NAME?` would move the reproduced cells away from Excel.
T-3610 decides how a cell whose Excel value varies between fills is scored; this cell's M8 exit row waits on it.
Unblock when T-3610 lands a scoring rule (then the cell is scored by it, and this task closes with no engine change)
or when a fresh fill reproduces `#NAME?` with a rule that separates it from the T-4642 refills.
