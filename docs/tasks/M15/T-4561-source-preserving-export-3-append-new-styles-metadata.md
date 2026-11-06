---
id: T-4561
title: "Source-preserving export 3: append new styles, metadata and the calcChain rule to patched packages (SPEC-0034 D3.3-D3.5)"
milestone: M15
lane: engine
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4183, T-4560]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A patched package can hold cells with styles the source did not have, dynamic-array anchors, and added or removed
formula cells:
- new style records are appended to `styles.xml`;
- `metadata.xml` follows the export's rules;
- calcChain is dropped with its relationship and content-type override when the set of formula cells changed.

This covers SPEC-0034 D3.3 (styles), D3.4 (dynamic arrays), D3.5 and E2.

## Context
SPEC-0034. The style table must be a prefix-stable extension of the source's, or the patch falls back.

## Acceptance criteria
- [ ] A cell given a new number format patches to an appended `xf` (and `numFmt`), and the other style records are
      unchanged (unit test).
- [ ] Adding a formula cell drops calcChain consistently, and changing only values keeps it (unit tests).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Structural edits.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
