---
id: T-4613
title: "Round-trip validator false positives: V2 on in-document # relationship targets, V4 on UTF-16 parts"
milestone: M15
lane: harness
status: blocked
priority: P2
size: S
tier: sonnet
depends_on: [T-4560]
---

## Goal
The package validator (SPEC-0033 D6) reports only real problems. It does not flag in-document relationship
targets or UTF-16 XML parts that Excel itself writes and reads.

## Context
In T-4560's corpus runs, the patched outputs copy these parts verbatim from the sources, and the validator still
fails them:
- V2: `"xl/drawings/drawing1.xml" rId1 targets the missing part xl/drawings/#…`
  (`roundtrip-calipers/pv_goldens/pv23_001/golden.xlsx`, 465 items in Calipers). A `#Sheet!A1` hyperlink target
  is a location in the document, not a part.
- V4: `customxml/item1.xml does not parse: not UTF-8` (`roundtrip-corpus-bench/33355/1_33355_input.xlsx`). XML
  may be UTF-16 with a BOM.

Code: `crates/rustsheet-eval/src/roundtrip/package.rs` (`validate`, `relationship_refs`, the part reader).

## Acceptance criteria
- [ ] A hyperlink relationship whose target starts with `#` is not resolved as a part (unit test).
- [ ] A UTF-16 part with a BOM is decoded before parsing, under the same byte budget (unit test).
- [ ] The corpus round-trip baselines are rerecorded, with fewer failing cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New validator checks.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
