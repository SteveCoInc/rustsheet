---
id: T-5005
title: "Round-trip R2 sheet-level scenarios: rename, add, delete and move a sheet (SPEC-0033 R2, SPEC-0034 D5.7)"
milestone: M15
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-4562]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
The round-trip suites score structural edits that need no M4: rename, add, delete and move a sheet. These are the
R2 scenarios of SPEC-0033, so the gains and drops of stage 5 are measured.

## Context
SPEC-0033 defines R2 (D1 table, the `stale` verdict, D7.2's rule that R2 drops explain themselves). It defers R2
scoring to M4 because of row and column edits. The sheet-level edits are already real `Txn` edits in the facade
(`crates/rustsheet/src/edit.rs`). The scenario table is `crates/rustsheet-eval/src/roundtrip/scenarios.rs`, and the
expected-delta rule is R1's, `roundtrip/r1.rs`.

## Acceptance criteria
- [ ] Scenarios R2/rename, R2/add, R2/delete and R2/move exist, each with its expected delta: a renamed sheet's name
      items, a deleted sheet's parts reported `OwnerDeleted`.
- [ ] The fixtures suite runs them, and the corpus suites run R2/rename and R2/delete. The baselines are recorded under
      a decision record if they add failing items.
- [ ] SPEC-0033 is edited to say sheet-level R2 is scored now: the Non-goals bullet "R2 implementation", the D1
      table's R2 row (the milestone column), and Rollout item 5's "R2 scoring follows M4".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row and column R2 (after M4).
