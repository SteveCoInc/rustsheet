---
id: T-4831
title: "Source-preserving export: add sharedStrings.xml to a patched package whose source has none"
milestone: M15
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-4514]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A patched save that needs its first shared string adds `xl/sharedStrings.xml`, with its relationship and
content-type override, instead of regenerating the package.

## Context
- T-4514's R1/E5 writes a new text. On a source with no `sharedStrings.xml`, `patch.rs` falls back with "new strings
  need a shared-string part that holds every string".
- E5's kept rates: poi 98.91%, corpus-bench 99.43%, calipers 99.944%. Many fixtures have no text at all.
- T-4561's `export/patch/package.rs` (`add_parts`) already adds parts with their links; the shared-string part is
  one more.

## Acceptance criteria
- [ ] A source without `sharedStrings.xml` given a text cell patches. The part holds the one string, `count` and
      `uniqueCount` are 1, and the relationship and override match what the regenerating export writes
      (integration test).
- [ ] Inline strings the model interned still fall back, as T-4560 rules (unit test).
- [ ] The round-trip baselines are rerecorded, and R1/E5 violations only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New styles (T-4623).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
