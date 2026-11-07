---
id: T-4943
title: "A pivot date item below 1 shows as a time (1904 serial 0 is 12:00:00 AM)"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot's date item below 1 shows as Excel shows it: in a 1904 workbook Excel's refresh shows serial 0 as `12:00:00 AM`, where the engine shows `1/1/1904`.

## Context
T-4807 gave every other date item Excel's `m/d/yyyy` (with ` h:mm` for a time part) label format (pivots.md "Item casing when cache and source disagree, and date items"), fixing the 1900 and 1904 refresh cases' P!A5:A10, the 1900 case's P!A4 and t4177-api-items-refresh P!A8. What remains (excel-graph, 2026-11-07):
- pivot-export/t4177-cache-dates-1904-refresh.export.json P!A4: Excel `12:00:00 AM`, the engine `1/1/1904` (a TF line, decision 0070).
Open: which format Excel uses for an item below 1 (`h:mm:ss AM/PM` fits this one cell), and whether a 1900-system item below 1 (serial 0, a time-only value) does the same; a probe should cover both. Code: `date_item_format` (crates/rustsheet-engine/src/pivot/layout/render.rs).

## Acceptance criteria
- [ ] The text of that cell matches Excel's, and pivots.md records the rule for items below 1 with probe evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
