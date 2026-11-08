---
id: T-5060
title: "t4618 Q!A1: a sheet-qualified use of an undefined name, P!N, is #NAME? in Excel and 1 ours"
milestone: M8
lane: engine
status: done
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
- [x] ~~t4618-order-after-active-backward Q!A1 is `#NAME?`~~ (the premise does not reproduce): Q!A1 is scored per
      decision 0358 against probes/t4642-backward-refill (1) and passes; no M8 cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Decision 0358 (owner ruling, 2026-11-08): a cell a later Excel fill of the same inputs superseded, or that later fills do not reproduce, is scored against that later fill. Spec suites name a `superseded` file (eval/suites/probes, gen-m7, gen-m9 `superseded.tsv`, 55 lines); `runner/prepare.rs` `apply_superseded` scores each line's cell against the same cell of the later golden sidecar, refusing a line whose later case is not a copy (`overrides::same_inputs`) or that cites another cell. The earlier sidecars stay committed. probes/t4618-order-after-active-backward Q!A1 is scored against the reproduced 2026-11-07 refill (1, as the engine gives) and passes, with no engine change: the 2026-11-06 `#NAME?` does not reproduce (syntax.md "Which place a loaded use took").
