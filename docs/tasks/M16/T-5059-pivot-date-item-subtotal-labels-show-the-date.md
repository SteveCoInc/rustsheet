---
id: T-5059
title: "Pivot date item subtotal labels show the date, not its serial (40 gen-pivot cells)"
milestone: M16
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A pivot's subtotal label for a date item shows the item as Excel does (`3/31/2024 Total`, `12/31/2022 Sum`,
`12/30/1900 18:00 Total`), not its serial (`45382 Total`).

## Context
- Found by the Phase 1 owners check (2026-11-07, main e560a1bb): 40 gen-pivot exit cells had no open M16 owner. All 40
  are the same text mismatch: the item part of a date item's subtotal label is the raw serial. T-4807 and T-4957 put date
  items in the source field's date number format but left "date subtotal labels" UNVERIFIED (T-4807 Outcome).
- Excel's labels include times (`12/31/2099 23:59 Total`, `2/29/2024 6:00 Total`), the 1900 leap-day item
  (`2/29/1900 Sum`) and 1904-looking serials (`1/1/1904 Total` is serial 1462 in the 1900 system).
- Cells:
  - gen-pivot/2251b0f5e5a256e1.case.json (3): P04!Q6, P04!T6, P04!W6
  - gen-pivot/36996978427905b2.case.json (3): P02!L35, P02!L40, P02!L45
  - gen-pivot/462f10551d480f9d.case.json (8): P04!M38, P04!M47, P04!M55, P04!M62, P04!M71, P04!M79, P05!M39, P05!M42
  - gen-pivot/a6d74990f3ae755e.case.json (6): P04!AB7, P04!O7, P04!R7, P04!T7, P04!W7, P04!Y7
  - gen-pivot/ca5c8fa99c13d747.case.json (2): P05!Q6, P05!S6
  - gen-pivot/ec473260bb2065b3.case.json (18): P09!AD7, P09!AE7, P09!AI7, P09!AJ7, P09!AL7, P09!AM7, P09!AQ7, P09!AR7,
    P09!AV7, P09!AW7, P09!AY7, P09!AZ7, P09!R7, P09!V7, P09!W7, P09!Y7, P09!Z7, P11!S6
- Code: the subtotal-label builder of the pivot renderer; reuse the date-item formatting T-4807 added (one definition).

## Acceptance criteria
- [ ] The 40 cells above match Excel's text; no other gen-pivot or pivot-corpus cell regresses.
- [ ] docs/excel-semantics records the subtotal-label rule (the T-4807 UNVERIFIED edge).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date page-item selections and seconds (T-4807's other UNVERIFIED edges).
