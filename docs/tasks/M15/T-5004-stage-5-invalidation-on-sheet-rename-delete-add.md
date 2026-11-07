---
id: T-5004
title: "Stage 5: invalidation on sheet rename, delete, add and move and on modelled-feature edits (SPEC-0034 D5.5)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-5001, T-5002]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
What an edit makes stale is dropped and reported, never kept: SPEC-0034 D5.5's rows for sheet rename, delete, add
and move, and for edits to modelled features (merges, conditional formats, tables, hyperlinks, column or row
properties).

## Context
Decision 0275 D4 says to drop and report, never keep stale. The triggers come from the pristine-against-live diff (D2).
The reasons are SPEC-0033 D7's `StaleAfterStructuralEdit` and `OwnerDeleted`. Sheet-name matching reuses the engine's
sheet-name quoting rules, with no second definition (directive 3).

## Acceptance criteria
- [ ] Each D5.5 row except the M4 one has a unit test: what is dropped, its reason, and what is kept.
- [ ] Through T-5005's R2 scenarios, each drop scores `dropped` (pass), and no kept item scores `stale`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row and column edits (T-5006).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
