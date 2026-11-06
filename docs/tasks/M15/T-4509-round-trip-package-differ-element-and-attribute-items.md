---
id: T-4509
title: "Round-trip package differ: element and attribute items, defaults table, unordered paths, normalization tags (SPEC-0033 D4)"
milestone: M15
lane: harness
status: done
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

Carried from PR #3529's review:
- give `Package::read` a cumulative inflated-byte budget across parts (security), before T-4173 brings untrusted
  corpora;
- use an id→position map instead of `first_ref`'s scan in `assign_keys` (security);
- add `Status:` and `Tasks:` lines to the "Round trips" section of xlsx-storage.md (fidelity).

## Acceptance criteria
- [x] Identity: `P(src, src)` is empty for every case of the fixtures round-trip suite.
- [x] Invariance: a crafted copy with re-prefixed namespaces, reordered unordered collections and default attributes
      written out compares `kept`.
- [x] Sensitivity: one lost `ext`, one lost element and one lost attribute are each found with the right key and
      verdict.
- [x] The fixtures baseline is updated, and the report lists lost elements by path.
- [x] `make check` green; reviewers approve.

## Out of scope
- Semantic items (T-4513).
- R1 (T-4514).

## Outcome
`roundtrip/xmldiff.rs` diffs every kept XML part pair as SPEC-0033 D4 says:
- namespace-URI names (D4.1);
- ordered children, except the cited unordered paths and `extLst`'s `ext[uri]` (D4.2);
- a schema-defaults table with CT_ citations, plus xsd:boolean equivalence (D4.3);
- `mc:Ignorable` as a set of resolved URIs;
- whitespace per D4.7;
- `candidate-N1`…`N12` tags, never applied (D4.4);
- lost elements absorbing their descendants;
- `sheetData` and `sst` children left to T-4513;
- 500 items per part, then a `<truncated>` item.

Binary parts compare by SHA-256 (D4.5).

Carried from PR #3529's review:
- `Package::read` reads every part once under one inflated-byte budget, using the snapshot reader's new
  `read_part_bytes_capped`;
- `assign_keys` uses an id→position map;
- xlsx-storage.md "Round trips" has `Status:` and `Tasks:` lines.

A case's checked count is now the number of source items compared. `roundtrip-fixtures`: 16,616 items, 3,167 known
failures. The new losses include:
- `pageMargins` and `sheetFormatPr` lost from every sheet;
- `fileVersion` and `tableStyles` lost;
- the theme's format scheme replaced by the template's;
- docProps rewritten (N2, N3);
- window geometry lost (N11).

Tests:
- identity on every fixture part;
- invariance under re-prefixed namespaces, reordered unordered collections, written-out defaults and boolean
  spellings;
- sensitivity to a lost `ext`, a lost element and a lost attribute;
- the semantic-subtree skip;
- candidate tags.

No cell eval delta.

