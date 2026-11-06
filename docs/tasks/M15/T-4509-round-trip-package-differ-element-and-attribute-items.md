---
id: T-4509
title: "Round-trip package differ: element and attribute items, defaults table, unordered paths, normalization tags (SPEC-0033 D4)"
milestone: M15
lane: harness
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4172]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
The round-trip differ compares the XML of every matched part pair as SPEC-0033 D4 says, and scores `element` and
`attribute` items, so that content the feature model does not know (unknown `extLst` entries, unknown elements and
attributes, new Excel features) is scored rather than ignored.

## Context
SPEC-0033 D2 and D4. T-4172 lands the suite kind and part-level items; this task adds the level below. The differ:
- matches names by namespace URI and local name, never by prefix;
- compares `mc:Ignorable` as a set;
- compares children in order except at the unordered paths, which are data with a schema citation per path;
- applies the attribute-defaults table, which is data with an ECMA-376 or MS-OI29500 citation per entry;
- tags candidate normalizations N1–N12 `candidate-N<k>`, and normalizes only items whose evidence row exists
  (T-4503);
- lets a lost element absorb its descendants and attributes, reported once with a count;
- treats whitespace as D4.7 says;
- caps the items reported per case, with the cap reported.

Read parts through the bounded reader T-4172 uses.

## Acceptance criteria
- [ ] Identity: `P(src, src)` is empty for every case of the fixtures round-trip suite.
- [ ] Invariance: a crafted copy with re-prefixed namespaces, reordered unordered collections and default attributes
      written out compares `kept`.
- [ ] Sensitivity: one lost `ext`, one lost element and one lost attribute are each found with the right key and
      verdict.
- [ ] The fixtures baseline is updated, and the report lists lost elements by path.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Semantic items (T-4513).
- R1 (T-4514).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
