---
id: T-4804
title: "GETPIVOTDATA: several data fields, the Values field, blank and empty data_field, one- and no-record aggregates, overflow (T-2085 evidence)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
depends_on: []
---

## Goal
GETPIVOTDATA matches Excel on pivots with several data fields, on blank and empty `data_field` arguments, on aggregates over one record or none, and on totals that overflow.

## Context
T-2085 probed SPEC-0015 unknowns 1, 2, 8, 15 and 16 (docs/excel-semantics/web.md "GETPIVOTDATA on other pivot shapes (T-2085)", the "Several data fields", "Blank and empty
data_field", "Aggregates over one record or none" and "Overflow" paragraphs). The engine
(`functions/lookup_and_reference/getpivotdata.rs`, `pivot_index.rs`) answers `#REF!` for most of these, and the grand
total for a blank cell outside the pivot, where Excel gives `#REF!`.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-data-fields Q!A1:A5, Q!A8, Q!A12, Q!A13, Q!A15, Q!A17, Q!C6, Q!F6, Q!G5, Q!G6, Q!H6,
  Q!I5, Q!I6, Q!J6, Q!K1:K3, Q!L1:L3

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] SPEC-0015's resolution rules and web.md's "Ours" line say what the engine now does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Item kinds and argument errors (T-4805), and pivot shapes and shown values (T-4806).
