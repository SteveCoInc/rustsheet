---
id: T-4567
title: "Spec: preservation under structural edits - ownership matrix, closure rule, fragment slots, invalidation table and limits (SPEC-0034 stage 5)"
milestone: M15
lane: research
status: in-progress
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
- [ ] SPEC-0034 gains its stage 5 section, and the planner files its implementation tasks.
- [ ] Reviewers approve.

## Out of scope
- Implementation.
