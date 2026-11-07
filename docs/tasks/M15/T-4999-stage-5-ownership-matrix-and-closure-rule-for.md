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
One ownership table, one closure computation and one coupling-group detector decide, for every part of the source
package, whether a regenerated save keeps it. Every part that is not kept is reported.

## Context
SPEC-0034 D5.1 (the matrix), D5.2 (coupling groups, the closure rule, the `c:pivotSource` exclusion) and D5.6 (limits,
with the deterministic drop order). The table lives in `export/ownership.rs`, keyed by relationship type and then by
content type, never by part name. This task computes the decision and reports it. Writing the parts is T-5000 onward.
The package plumbing already exists: `PartNames`, `parse_relationships` and `resolve_target`
(crates/rustsheet-xlsx/src/import/package.rs).

## Acceptance criteria
- [ ] `export/ownership.rs` holds the D5.1 table, one row per relationship type listed there.
- [ ] `policy_of(rel_type, content_type, reached_from)` gives the unknown defaults (package/workbook-reached, or
      sheet-reached).
- [ ] `closure(source, root)` follows internal relationships only, within the D5.6 depth and size limits.
- [ ] The legacy-drawing and slicer coupling groups are detected per sheet, as D5.2 defines them.
- [ ] Each source part of a Regenerated save with a source gets a decision: kept, or dropped with a SPEC-0033 D7 reason.
      The drops appear in the SaveReport (T-4562).
- [ ] Limits are applied in source part-name order, so the result is deterministic.
- [ ] Unit tests cover:
  - each policy, and the unknown defaults both ways;
  - a closure that reaches a `Modelled` part, and a chart holding `c:pivotSource`;
  - a shared part, and an external target;
  - each coupling group;
  - each limit.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing kept parts (T-5000, T-5002, T-5008 to T-5010).
- Fragments (T-5001, T-5003).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
