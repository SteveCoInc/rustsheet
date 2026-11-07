---
id: T-4642
title: "A qualified use of a provisional name place reads #NAME? in probes/t4618-order-after-active-backward Q!A1"
milestone: M8
lane: api
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Find why Excel leaves probes/t4618-order-after-active-backward Q!A1 (`=P!N` on Q, loaded before `N` = `=1` is added
through the API scoped to P) `#NAME?`, where every other qualified use of the T-4618 probes reads the added name, and
make the engine match.

## Context
- Evidence (excel-graph 2026-11-06): tabs P, Q, R with R active (R holds `=1+1`). P holds `=N`, `=N+1`, Q holds `=P!N`.
  Q!A1 is `#NAME?`. In probes/t4618-order-wraps-after-active (Q active and holding `=1+1`, P's `=N`, R's `=P!N`),
  R!A1 is 1. In probes/t4049-sheet-api-name-plain, T!A1 `=S!N` after S's `=N` is 1.
- docs/excel-semantics/syntax.md "Which place a loaded use took" (T-4618): the load-order place model explains every
  bare-use cell, and Q!A1 is its one exception. Under it, Q's `=P!N` fixes P's provisional place to the workbook. So
  either a qualified use bound to a fixed place reads `#NAME?` and something else makes the others read 1, or the
  order of the non-active sheets differs from tab order in a way the other probes do not show.
- Ours: qualified uses are never pinned (`Model::bind_added_sheet_name`), so Q!A1 reads 1.
- Probe one variable per case: the same book with Q and R swapped; with a fourth tab; with Q's `=P!N` also
  typed before the add.

## Acceptance criteria
- [ ] probes/t4618-order-after-active-backward Q!A1 passes, or the probes show what it depends on, recorded in
      syntax.md "Which place a loaded use took".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unqualified uses on the scoped sheet (T-4618).
