---
id: T-4167
title: "Spec: the xlsx round-trip contract (scenarios R0-R2, semantic and package equivalence, expected-delta rule, SaveReport)"
milestone: M15
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
SPEC-0033 defines what an xlsx round trip must preserve and how we measure it, so every later M15 task has one
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
- [x] A merged spec in docs/specs/ covers each item above, with the open research questions RQ2 and RQ5 named as
      dependencies.
- [x] Reviewers approve.

## Out of scope
- Implementation.

## Outcome
docs/specs/SPEC-0033-xlsx-round-trip-contract.md defines:
- the scenarios R0, R1 (standard edit set E1–E6, run singly and together) and R2 (keyed now, scored with M4);
- fidelity items and their verdicts (`kept`, `normalized`, `changed`, `lost`, `dropped`, `stale`, `added`, `n/a`);
- semantic (S) equivalence, the gate, with resolved indexes and styles;
- package (P) equivalence, the diagnostic, with a namespace-aware diff, unordered paths, a defaults table and an
  evidence-gated normalization list N1–N12;
- the expected-delta rule;
- the package validator V1–V8;
- the `SaveReport` API;
- the `roundtrip` suite kind with its baseline and exception format.

Dependencies:
- RQ2 is filed as T-4503 (blocked).
- RQ5 is T-4166.
- Decision 0306 records the owner's ruling that this session works M15 tasks ahead of Phase 1, one claimed task at
  a time.

No eval delta (docs only).
