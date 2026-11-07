---
id: T-4690
title: "Package validator on flaws the source package already has (an Override for a missing part, a dangling r:id, a beta-namespace sst): rule whether the save repairs them or V1, V4 and V7 skip them (SPEC-0033 D6)"
milestone: M15
lane: harness
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-4503]
---

## Goal
A `package-invalid` verdict on a round-trip output means the save broke the package. The rule for a flaw the
source package already had is written in SPEC-0033 D6, and the validator or the export follows it. Either the save
repairs the flaw, as Excel's own save does, or the check does not count a flaw the source already has.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main ebe81baf6, calipers, corpus-bench and poi fetched). These
  roundtrip.json V1, V4 and V7 items fail outputs whose sources have the same flaw. A scratch check of each source
  zip found the part missing, or the relationship id absent from the source's `.rels`:
  - V1, "an Override names the missing part", where the source has the Override and not the part:
    - roundtrip-calipers/pv_goldens/pv23_005/golden.xlsx: `xl/metadata`.
    - roundtrip-poi/64508.xlsx, 64667.xlsx and bug62181.xlsx: `xl/theme/theme1.xml`.
    - roundtrip-poi/65016.xlsx: `.xml`.
    - roundtrip-poi/bug63189.xlsx: `xl/drawings/drawing2.xml`.
    - roundtrip-poi/picture.xlsx: `docprops/custom.xml`, `xl/connections.xml` and `xl/theme/theme1.xml`.
  - V4, "r:id ... has no relationship", where the source's `.rels` lacks the id too:
    - roundtrip-calipers/pv_goldens/pv23_005 and pv23_036: `xl/workbook.xml` rId5. This is Google Sheets'
      `go:sheetsCustomData`.
    - pv23_060: rId9.
    - pv23_066: rId18.
    - roundtrip-corpus-bench/132-4/1_132-4_input.xlsx: `xl/externallinks/externallink1.xml` rId1.
  - V7, roundtrip-poi/sample-beta.xlsx Sheet1!A1:A10, "uses shared string n of 0". The source's sharedStrings part
    uses the Excel 2007 beta namespace (`.../office/excel/2006/2`, `<sstItem>`), so the validator counts 0 strings.
    Whether the importer reads those strings is not checked: poi/sample-beta.xlsx is a pending eval case.
- SPEC-0033 D6 says "Any failure is a `package-invalid` verdict on the case, which always fails". It does not say
  whether an inherited flaw counts. Excel's own save probably drops a dangling Override or r:id. T-4503 probes what
  Excel rewrites on every save (N1-N12), which settles whether the export should repair the flaw. That is why this
  task depends on it.
- Code: crates/rustsheet-eval/src/roundtrip/package.rs (`validate`); the export's content-types and relationship
  writers in crates/rustsheet-xlsx/src/export/.
- T-4613 handles the other validator false positives (V2 on `#` hyperlink targets, V4 on UTF-16 parts). It does not
  cover these items, and they do not overlap.

## Acceptance criteria
- [ ] SPEC-0033 D6 states the rule for a flaw the source package already has, citing T-4503's evidence on what
      Excel's save does with it.
- [ ] The validator or the export follows the rule, with a unit test per check (V1 Override, V4 r:id, V7 beta sst)
      on a source that has the flaw and on one that does not.
- [ ] After `rustsheet-eval roundtrip --update-baseline` with every suite fetched, the items above leave
      eval/runs/latest/roundtrip.json, and no new F line appears in any eval/baseline/roundtrip-*.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- V5 (roundtrip-poi/duplicate-filename-case-insensitive.xlsx) and V6 (xlsx-corrupted.xlsx, and xssf-enum.xltx.xlsx's
  sheet name with `_x000a_`). Each needs its own look at what the source holds.
- The V2 and UTF-16 V4 items (T-4613).
- Value items (T-4689).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session. It also waits on T-4503.
