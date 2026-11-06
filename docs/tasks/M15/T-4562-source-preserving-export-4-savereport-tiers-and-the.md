---
id: T-4562
title: "Source-preserving export 4: SaveReport tiers and the round-trip dropped verdict (SPEC-0034 D4, SPEC-0033 D7)"
milestone: M15
lane: engine
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4183, T-4561]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
`save_xlsx_with_report` returns the tier (`Verbatim`, `Patched`, `Regenerated { reason }`) and every drop. The
round-trip suite's `dropped` verdict reads it (SPEC-0034 D4, SPEC-0033 D7).

## Context
SPEC-0033 D7 defines `SaveReport`, `Dropped` and `DropReason`, and SPEC-0034 D4 adds the tier. The facade's
`save_xlsx` and `to_xlsx_bytes` keep their signatures.

## Acceptance criteria
- [ ] The facade gains `save_xlsx_with_report` and `to_xlsx_bytes_with_report`, with Python `save_xlsx(...,
      report=True)`.
- [ ] Round-trip items named by the report with an accepted reason score `dropped`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Stage 4 preservation under regeneration.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
