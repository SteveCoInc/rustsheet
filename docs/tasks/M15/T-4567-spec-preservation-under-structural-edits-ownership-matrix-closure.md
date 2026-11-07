---
id: T-4567
title: "Spec: preservation under structural edits - ownership matrix, closure rule, fragment slots, invalidation table and limits (SPEC-0034 stage 5)"
milestone: M15
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4183]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A spec section (SPEC-0034 D5, stage 5) defines how a save that must regenerate still preserves what the model does
not hold:
- the package ownership matrix (part pattern and relationship type to `Modelled`, `Replay`, `ReplayIfClean`,
  `Active`, `Drop`);
- the closure rule;
- fragment slots in regenerated parts, keyed by schema slot and `ext` uri;
- the invalidation table on structural edits;
- the preservation limits.

## Context
Decision 0275 (D4–D6) and docs/research/xlsx-round-trip-2026-11-04.md §5 layers 1–4. Prior art: Mog's
`write/package_ownership.rs`, GridCraft's `passthrough.rs`. T-4183 scoped SPEC-0034 to the source-preserving tiers,
which cover every save with no structural change, and moved this to stage 5. It needs M4's reference shifting for
range-bound content. SPEC-0033 D7's `ClosureReachesRegenerated` and `Policy` drops are defined here.

## Acceptance criteria
- [x] SPEC-0034 gains its stage 5 section, and the planner files its implementation tasks.
- [x] Reviewers approve.

## Out of scope
- Implementation.

## Outcome
SPEC-0034 gains D5, "Stage 5: preserving under regeneration". A regenerated save with a source becomes "regenerate,
then keep". Everything is read from the retained source bytes at save time, so the model and the importer hold no
preservation fields.
- **D5.1 Ownership matrix.** One table keyed by relationship type, with the policies `Modelled`, `Replay`,
  `ReplayIfClean`, `Active` and `Drop`. Unknown types follow the closure rule. Chartsheets, dialogsheets and macrosheets
  are a named gap.
- **D5.2 Closure rule.** It follows internal relationships only. A root is kept iff its closure reaches nothing modelled,
  dropped, or owned by another sheet. A part shared by two roots is written once.
- **D5.3 Writing.** Source names are kept, with collisions renamed. The owner `.rels` and `[Content_Types].xml` are
  re-derived, relationship ids are fresh, and attach points are remapped.
- **D5.4 Fragment slots.** Each writer declares an owned set; everything else is a fragment, emitted at its schema
  position. Fragments are classified as relationship-, range-, sheet-name- or index-bound, or inert. `x15ac:absPath` is
  kept, following decision 0275 D6 over the research doc's "drop", with an option to strip it.
- **D5.5 Invalidation table.** Drop and report, never keep stale (decision 0275 D4). The row-and-column row is
  provisional on M4's API.
- **D5.6 Limits.** Depth 16, 1,000 parts per closure, fragment depth 256, 10,000 fragments per part. A limit drops what
  it covers and never fails the save.
- **D5.7 Measurement** (R2 sheet-level scenarios, re-save probes) and **D5.8** (what Excel behaviour stays UNVERIFIED).
- SPEC-0033 D7's `Policy` and `ClosureReachesRegenerated` are now defined.

Tasks filed, all M15 and `blocked` (decision 0306):
- T-4999: matrix and closure;
- T-5000: package- and workbook-scoped parts;
- T-5001: worksheet fragments;
- T-5002: sheet-anchored parts with their attach elements;
- T-5003: workbook and styles fragments, and the absPath option;
- T-5004: invalidation on sheet and modelled-feature edits;
- T-5005: R2 sheet-level scenarios;
- T-5006: stage 5b, row and column invalidation after M4;
- T-5007: spec for stage 5c, the shifter.

Order: T-4562, then T-4999 and T-5005, then T-5000, T-5001 and T-5003, then T-5002, then T-5004. T-4950 now points at
T-5002 and T-5004.
