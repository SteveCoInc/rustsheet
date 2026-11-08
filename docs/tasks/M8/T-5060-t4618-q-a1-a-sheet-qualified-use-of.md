---
id: T-5060
title: "t4618 Q!A1: a sheet-qualified use of an undefined name, P!N, is #NAME? in Excel and 1 ours"
milestone: M8
lane: engine
status: blocked
priority: P1
size: S
tier: opus
depends_on: [T-3610]
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

## Blocked (2026-11-08)
No engine change: the cell is not an engine bug. docs/excel-semantics/syntax.md "Which place a loaded use
took" (T-4642) records that Q!A1's `#NAME?` (2026-11-06 fill) does not reproduce: probes/t4642-backward-refill and
-refill-b, the same input bytes (sha256 f800e58a…), read 1 on 2026-11-07, as does every one-variable variant
(t4642-qualified-after-active, -fourth-tab-before, -fourth-tab-after, -qualified-entered, -no-bare-use), and a
qualified use reads the new name in every other probe. Making `=P!N` `#NAME?` here would regress those cells and
follow a value Excel itself did not repeat (directive 1 asks for Excel's behavior, and the reproduced behavior is
1). The cell is a fill-variation cell, and T-3610 already owns how the eval scores it (its Context lists this exact
cell). This task stays open, blocked on T-3610, so the M8 exit cell keeps an M8 owner (scripts/exit_owners.py); once
T-3610's decision is applied it closes with no engine change, unless a new fill gives evidence of a rule.
