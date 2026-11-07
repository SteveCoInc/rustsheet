---
id: T-5004
title: "Stage 5: invalidation on sheet rename, delete, add and move and on modelled-feature edits (SPEC-0034 D5.5)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-5001, T-5002, T-5005, T-5008, T-5009]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
What an edit makes stale is dropped and reported, never kept. This covers every SPEC-0034 D5.5 trigger except the
*(M4)* row: feature, notes, pivot or table and style-table edits, and sheet rename, delete, add and move.

## Context
- Decision 0275 D4 says drop and report, never keep stale.
- The triggers are D5.5's closed set, each detected from the pristine-against-live diff (D2).
- The reasons are SPEC-0033 D7's `StaleAfterEdit`, `StaleAfterStructuralEdit` and `OwnerDeleted`. `StaleAfterEdit` is
  new in D7, and T-4562 implements it with the rest.
- Sheet-name matching reuses the engine's `quote_name` (public) and `unquote_sheet_prefix`, which this task makes
  public. There is no second copy in xlsx (directive 3).
- The text scans are bounded per D5.6.

## Acceptance criteria
- [ ] The trigger set is defined once, next to the matrix.
- [ ] Each D5.5 row except the M4 one has a unit test: what is dropped, its reason and `sheet`, and what is kept.
- [ ] Through T-5005's R2 scenarios and R1's E1 to E6, each drop scores `dropped` (pass), and no kept item scores
      `stale`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row and column edits (T-5006).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
