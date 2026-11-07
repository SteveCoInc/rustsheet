---
id: T-4832
title: "Source-preserving export: break a shared-formula group whose master or member is edited, instead of regenerating (SPEC-0034 D3.4)"
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
Editing a cell of a shared-formula group is patched, not regenerated. The group is broken as SPEC-0034 D3.4 says:
an edited member leaves the group, and when the master changes every member is written with its own formula.

## Context
- T-4514's R1/E2 (the first formula becomes `=1+1`) loses 2–4% of the corpora's items (calipers 96.47%, poi
  97.73%, corpus-bench 99.45%).
- `patch.rs` falls back on "a changed formula in a shared-formula group" and "a cleared cell in a shared-formula
  group".
- D3.4 says a broken group's members are written with their own formulas, and an edited member alone leaves the
  group.

## Acceptance criteria
- [ ] An edited member is written whole, and the other members keep `t="shared"` (integration test).
- [ ] A changed or cleared master writes every member of its group with its own formula, and no `si` is left
      dangling (integration test).
- [ ] The round-trip baselines are rerecorded, and R1/E2 violations only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New styles (T-4623).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
