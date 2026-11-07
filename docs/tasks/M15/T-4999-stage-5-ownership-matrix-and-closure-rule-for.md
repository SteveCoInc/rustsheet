---
id: T-4999
title: "Stage 5: ownership matrix and closure rule for regenerated saves (SPEC-0034 D5.1, D5.2, D5.6)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-4562]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
One ownership table and one closure computation decide, for every part of the source package, whether a regenerated
save keeps it. Every part not kept is reported.

## Context
SPEC-0034 D5.1 (the matrix), D5.2 (the closure rule) and D5.6 (the limits). The table lives in `export/ownership.rs`,
keyed by relationship type and then content type, never by part name. This task computes the decision and reports
it. Writing the kept parts is T-5000 onward. The package plumbing already exists: `PartNames`,
`parse_relationships`, `resolve_target` (crates/rustsheet-xlsx/src/import/package.rs).

## Acceptance criteria
- [ ] `export/ownership.rs` holds the D5.1 table, with one row per relationship type listed there, and `policy_of(rel_type,
      content_type)`, whose unknown default follows D5.2.
- [ ] `closure(source, root)` follows internal relationships only, within the D5.6 depth and size limits. It returns the
      parts, or the limit that was hit.
- [ ] For a Regenerated save with a source, each source part gets a decision (kept, or dropped with SPEC-0033 D7's
      reason). The parts that are dropped appear in the SaveReport (T-4562).
- [ ] Unit tests cover each policy, the unknown default both ways, a closure that reaches a `Modelled` part, a shared
      part, an external target, and each limit.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing kept parts (T-5000, T-5002); fragments (T-5001, T-5003).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
