---
id: T-5006
title: "Stage 5b: invalidation on row and column insert and delete (SPEC-0034 D5.5, after M4)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-5004]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
After a row or column insert or delete on sheet S, the range-bound fragments of S and the `ReplayIfClean` parts
anchored to S are dropped and reported (`StaleAfterStructuralEdit`), never kept stale. This is SPEC-0034 D5.5's
*(M4)* row.

## Context
It needs M4's structural-edit API (docs/milestones.md M4). Today row and column edits exist only behind the
`harness_row_insert` feature (decisions 0076, 0098), so the D5.5 row is provisional and is re-checked against M4's
spec when that lands. T-5004 builds the invalidation machinery. This task adds the trigger.

## Acceptance criteria
- [ ] The trigger fires for every row and column edit of M4's API (unit tests).
- [ ] R2 row and column scenarios score the drops `dropped`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shifting instead of dropping (T-5007).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275), and it depends on M4's structural-edit API (row and column insert and
delete, `RefMap`). Unblock when docs/status.md has the line `**Phase 1:** exited <date>` and M4's structural-edit API
exists. An owner-directed session may claim it earlier (decision 0306), once that API exists.
