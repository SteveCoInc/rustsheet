---
id: T-4183
title: "Spec: package ownership matrix, closure rule, preservation limits and invalidation table for xlsx round trips"
milestone: M15
lane: research
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4167]
---

## Goal
SPEC-NNNN defines how the xlsx crate preserves what it does not model, so stages 1–3 of M15 can be built as small
tasks.

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §5 (decision 0275, D3–D6). The spec covers:
- bounded `SourcePackage` retention: the new `ImportLimits` for kept bytes, parts, relationships and closure hops;
- the ownership matrix: one table of part patterns and relationship types against the policies `Modelled`,
  `Replay`, `ReplayIfClean`, `Active` and `Drop`, with the default for unknown relationship types;
- the closure rule;
- rId and part-name remapping;
- fragment capture by schema slot and ext URI, with r:id, range-bound and sheet-name-bound classification;
- `mc:Ignorable` and AlternateContent policy;
- the invalidation table (drop and report on structural edits, D4);
- VBA into macro-enabled files only (D5);
- `x15ac:absPath` (D6);
- append-only shared strings and styles, verbatim replay of untouched sheets, and the cell patcher with its
  cross-check against regeneration (D3).

Prior art, cited in the plan:
- Mog `write/package_ownership.rs`;
- GridCraft `crates/xlsx/src/passthrough.rs`;
- umya lazy sheets;
- Formualizer span patching.

## Acceptance criteria
- [ ] A merged spec in docs/specs/.
- [ ] A task breakdown of stages 1–3 filed from it by the planner.
- [ ] Reviewers approve.

## Out of scope
- Implementation.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
