---
id: T-4183
title: "Spec: package ownership matrix, closure rule, preservation limits and invalidation table for xlsx round trips"
milestone: M15
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4167]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
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
- [x] A merged spec in docs/specs/.
- [x] A task breakdown of stages 1–3 filed from it by the planner.
- [x] Reviewers approve.

## Out of scope
- Implementation.

## Outcome
The spec is docs/specs/SPEC-0034-xlsx-source-preserving-export.md, the source-preserving export. It chose the
fastest route to the owner's 99% goal (decision 0275 D3):
- the facade retains the source bytes;
- a save-time diff (re-import the source, compare typed models) sorts a save into three tiers:
  - Unchanged: the source is written verbatim;
  - Content: a streaming patch of the changed `<c>` elements, with shared strings and styles appended
    index-stably, a calcChain rule and shared-formula breaking;
  - Structural: today's regeneration.

The ownership matrix, closure rule and fragment slots of decision 0275 become stage 4, preservation under
regeneration, after M4.

Implementation tasks: T-4559 (retain the source, verbatim tier), T-4560 (cell patcher, shared-string
append), T-4561 (styles append, metadata, calcChain), T-4562 (SaveReport tiers). No eval delta (docs
only).

