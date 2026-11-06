---
id: T-4559
title: "Source-preserving export 1: retain the source and write it back verbatim when nothing changed (SPEC-0034 D1, D2)"
milestone: M15
lane: engine
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4183]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
`rustsheet::Workbook` keeps the bytes it was loaded from. Saving a workbook whose cells and structure are unchanged
since load writes every part of the source back byte for byte (SPEC-0034 D1, D2 "Unchanged"). Any other workbook
keeps today's regenerating export.

## Context
SPEC-0034. The save-time diff re-imports the retained source into a pristine `Model` and compares it with the live
one, using the typed model's own fields (D2.1). This first slice only needs the Unchanged decision. Classifying
Content against Structural, and acting on it, is T-T-4560.

R0 recalculates. A workbook whose recalculation reproduces every cached value is Unchanged, so in
`roundtrip-fixtures` such cases become fully kept. That is the first big step towards the owner's 99% goal.

## Acceptance criteria
- [ ] Load then save, with no edit, of a workbook whose recalculation changes no cached value gives the source's
      parts, every part byte-identical (unit test).
- [ ] One edited cell, or one changed cached value, falls back to regeneration (unit test).
- [ ] The round-trip baselines are rerecorded. Their kept-item rate rises, reported in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Patching (T-T-4560).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
