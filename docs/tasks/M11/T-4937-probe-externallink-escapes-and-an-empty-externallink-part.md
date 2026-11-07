---
id: T-4937
title: "Probe externalLink escapes and an empty externalLink part in Excel"
milestone: M11
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for three UNVERIFIED points of the externalLink round trip (T-4143, xlsx-storage.md "externalLink parts"):
1. Whether Excel opens a saved file whose externalLink part is an empty `<externalLink/>` (the export writes one for a
   DDE/OLE link or a missing part) without a repair prompt. If it does not, find the smallest part it accepts.
2. Whether Excel decodes `_xHHHH_` escapes in a cached `t="str"` cell's `<v>`. The import now assumes it does.
3. Whether Excel decodes them in externalLink attributes (`sheetName val`, `definedName name`/`refersTo`). The export
   escapes attributes with `escape_xml`, but the import does not decode them, so a linked sheet named like `a_x0041_b`
   would not round-trip.

## Context
Raised by the Excel-fidelity review of PR #3957 (T-4143). Workbook sheet names on main use the same escaping pattern,
so point 3 may also concern `<sheet name>`. Use the `excel-probe` skill. The `rustsheet-oracle` path that opens
uploaded files applies.

## Acceptance criteria
- [ ] xlsx-storage.md states the result of each point with its evidence, and the UNVERIFIED marks go.
- [ ] Any mismatch with the engine gets a follow-up task, or a fix if it is size S.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Modelling DDE or OLE links.
