---
id: T-4674
title: "Match probes/t4621-api-sheet-name-body* column E (bare own-sheet uses of a sheet-scoped API add) once T-4618 lands"
milestone: M8
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4618]
---

## Goal
The bare own-sheet uses that T-4621's probes load before a sheet-scoped API add read what Excel reads: the new name
on some sheets and active sheets, `#NAME?` on others.

## Context
- T-4621's probes (excel-graph 2026-11-06; tabs First, S2, Last; adds SD, SW, TW on S2 and LD on Last): column E holds
  `=SD`, `=SW`, `=TW` on S2 and `=LD` on Last, typed before the adds. The engine leaves all of them `#NAME?`. Excel
  resolves some, depending on the active sheet. Off cells (Excel resolves, ours `#NAME?`):
  - probes/t4621-api-sheet-name-body Last!E2 (S2 active; Excel 2).
  - probes/t4621-api-sheet-name-body-first S2!E1:E4 and Last!E2 (First active; Excel 1, 93, 94 and 2; S2!E2 is empty).
  - probes/t4621-api-sheet-name-body-last S2!E1:E4 (Last active; Excel 1, 93, 94).
- T-4618's open PR (#3687, "Which place a loaded use took" in syntax.md) models this with load order (the active sheet
  first, then tab order) and own or provisional places. Worked by hand, it predicts all 8 cells and the 4 `#NAME?`
  ones (S2!E1, E3, E4 with S2 active; Last!E2 with Last active). Once T-4618 merges, these cells may simply pass.

## Acceptance criteria
- [ ] After T-4618 lands, `make eval-show` on the three cases: every column-E cell passes, or the
      mismatches are explained in syntax.md and owned by a task.
- [ ] `make eval-baseline` records the newly passing cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column D (qualified and INDIRECT uses) already passes (T-4621).
