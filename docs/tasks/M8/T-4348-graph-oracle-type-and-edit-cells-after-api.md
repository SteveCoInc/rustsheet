---
id: T-4348
title: "Graph oracle: type and edit cells after api_names, then probe a name typed after a sheet-scoped add"
milestone: M8
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A case can type or edit cells after its `api_names` are defined, and the open questions of T-4049 about a name added
after load are answered by the oracle.

## Context
- The excel-graph backend (`tools/src/rustsheet_tools/oracle/backends.py`, `recalc`) types the typed entries and
  replays the sheet ops before it adds `api_names`; the engine adapter (`crates/rustsheet-eval/src/engine.rs`,
  `define_api_names`) follows the same order. Neither can type or edit a cell after the add.
- T-4049 (docs/excel-semantics/syntax.md "A sheet-scoped name added after load") left these UNVERIFIED for that reason:
  `=N` typed on S after a sheet-scoped N is added (ours: 1, binding by its text); another cell on S edited after the
  add (does it re-bind a pinned `=N`?); `=T!N` typed on S, with no local N on T, when N is added to S (ours moves it
  to `S!N`); a typed `=[0]!Missing` (ours shows `=Missing`); a later workbook-scoped add over a pinned use;
  conditional-format and data-validation formulas on S.
- A spec shape such as an `after_api_names` list of typed entries, replayed by both the backend and the adapter in
  the same order.

## Acceptance criteria
- [ ] A case can type cells after its `api_names` (backend and engine adapter alike), documented in spec2xlsx's
      docstring and tested.
- [ ] Probes for the UNVERIFIED items above are filled and syntax.md is updated; the engine matches or each gap has
      an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel's Name Manager (T-1075).
