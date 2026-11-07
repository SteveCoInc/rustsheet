---
id: T-5000
title: "Stage 5: keep package- and workbook-scoped parts in a regenerated save (SPEC-0034 D5.3)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-4999]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A regenerated save keeps the source's package- and workbook-scoped unmodelled parts. These are customXml, custom
properties, the thumbnail, connections, people, web extensions and unknown part kinds, byte for byte.

## Context
SPEC-0034 D5.3:
- source names are kept unless they collide with a name the export writes;
- the owner `.rels` and `[Content_Types].xml` are re-derived, never copied;
- relationship ids are fresh.

These roots need no attach element in a regenerated part, so the task has no fragments. The round-trip suites score
the gain, as every fallback save of a workbook holding these parts loses them today.

## Acceptance criteria
- [ ] Every kept root and its closure (T-4999) is written, with its relationship and content type. Every dropped one is
      reported.
- [ ] A name collision renames the part and rewrites the `.rels` that target it (unit test).
- [ ] An unknown relationship type whose closure is clean survives a regenerated save (fixture test).
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps; findings go into docs/excel-semantics/xlsx-storage.md.
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sheet-anchored parts (T-5002); `Active` content into non-macro files (reported only).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
