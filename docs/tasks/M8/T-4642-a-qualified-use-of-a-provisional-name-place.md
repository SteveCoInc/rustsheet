---
id: T-4642
title: "A qualified use of a provisional name place reads #NAME? in probes/t4618-order-after-active-backward Q!A1"
milestone: M8
lane: api
status: done
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
- [x] probes/t4618-order-after-active-backward Q!A1 passes, or the probes show what it depends on, recorded in
      syntax.md "Which place a loaded use took".
- [x] `make check` green; reviewers approve.

## Out of scope
- Unqualified uses on the scoped sheet (T-4618).

## Outcome
Done 2026-11-07. The cell does not reproduce, so there is no engine change.
- **Probes** (excel-graph, 2026-11-07, all 7 pass with ours):
  - probes/t4642-backward-refill and -refill-b copy t4618-order-after-active-backward with identical sheets (the same
    input sha256 f800e58a…). Both read Q!A1 = 1. The two came from one upload, so they may be one fill.
  - The one-variable variants all read the qualified use as 1:
    - t4642-qualified-after-active (Q after the active R);
    - t4642-fourth-tab-before (a fourth tab Z);
    - t4642-fourth-tab-after (the use on Z);
    - t4642-qualified-entered (also typed before the add; Q!A1 and Q!A2);
    - t4642-no-bare-use.
  - P's bare uses stay `#NAME?` wherever the model says they should.
- **Finding:** the 2026-11-06 `#NAME?` at t4618-order-after-active-backward Q!A1 is variation between fills of the
  same bytes. No rule explains it: identical bytes now read 1, and every variant agrees. The original sidecar stays committed until T-3610 decides how such cells are scored,
  and Q!A1 stays a known failure. It is handed to T-3610 as a third variance cell (its Context and acceptance
  criteria list it). syntax.md "Which place a loaded use took" records the finding.
- **Eval delta:** probes gain 7 cases (all passing), known failures 6311 → 6311, 0 regressions.
