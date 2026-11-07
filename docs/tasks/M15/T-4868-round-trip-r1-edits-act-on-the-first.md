---
id: T-4868
title: "Round-trip R1 edits act on the first visible worksheet (facade: sheet visibility)"
milestone: M15
lane: harness
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: [T-4514]
---

## Goal
R1's standard edits act on the first visible worksheet, as SPEC-0033 D1 says, not on the first sheet in tab order.

## Context
- T-4514 picks the first sheet in tab order (crates/rustsheet-eval/src/roundtrip/r1.rs, `apply`), because the
  facade's `SheetMeta` has no visibility.
- A workbook whose first sheet is hidden is edited on that hidden sheet. How many corpus workbooks have one is
  unmeasured.

## Acceptance criteria
- [ ] The facade exposes a sheet's visibility (`SheetMeta` or a query), with a unit test.
- [ ] R1 edits the first visible worksheet, and is `n/a` when there is none (unit test).
- [ ] The count of corpus workbooks whose first sheet is hidden is in the Outcome, and the baselines are rerecorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Chartsheets and other sheet kinds.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
