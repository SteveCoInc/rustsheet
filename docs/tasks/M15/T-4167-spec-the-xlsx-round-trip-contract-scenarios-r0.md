---
id: T-4167
title: "Spec: the xlsx round-trip contract (scenarios R0-R2, semantic and package equivalence, expected-delta rule, SaveReport)"
milestone: M15
lane: research
status: blocked
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
SPEC-NNNN defines what an xlsx round trip must preserve and how we measure it, so every later M15 task has one
contract to test against.

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §4 and §6.1 (decision 0275). The spec fixes:
- the scenarios R0 (load, save), R1 (content edits, with a standard edit set) and R2 (structural edits);
- semantic (S) equivalence, the gate;
- package (P) equivalence, the diagnostic: a namespace-aware structural XML diff, an attribute-default table, rId,
  sst, style and part-name renumbering;
- the normalization list, each item with evidence that Excel itself rewrites it on save;
- the expected-delta rule for R1;
- fidelity items and their verdicts (kept, lost, changed, stale-kept, dropped-with-reason);
- the `SaveReport` API on the facade's `save_xlsx`.

Prior art: Calipers `internal/xlsxmodel` (semantic gate, per-case exceptions with a self-test) and `internal/compare`
(package normalizations); Mog `src/testing/xml_diff` (unordered paths, attribute defaults).

## Acceptance criteria
- [ ] A merged spec in docs/specs/ covers each item above, with the open research questions RQ2 and RQ5 named as
      dependencies.
- [ ] Reviewers approve.

## Out of scope
- Implementation.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
