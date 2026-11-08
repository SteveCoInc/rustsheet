---
id: T-5049
title: "Probe pivot date item labels: seconds, subtotal labels, page items, 1900 items below 1"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's shown text of a pivot date item's label is evidenced on the edges T-4807 left UNVERIFIED, each recorded in pivots.md.

## Context
T-4807 (PR #3952) gives a date item's label `m/d/yyyy`, with ` h:mm` for a time part (pivots.md "Item casing when cache
and source disagree, and date items"). Its reviewers listed what no probe covers yet:
- a time part with seconds (`4/17/2024 8:00:30`: does Excel show `8:00` or the seconds?);
- an ungrouped date field over a non-`m/d/yyyy` source (`yyyy-mm-dd`, `d-mmm-yy`, a time-only `h:mm` source); the
  `yyyy-mm-dd` one is answered: probes/t4717-pivot-cell-formats L5 reads `1/15/2024` (T-4717);
- a date item's subtotal label (`<item> Total`) with the date as an outer row field;
- a selected date page item (the page field's shown item);
- a 1900-system item below 1 (serial 0.5, a time-only value). The 1904 workbook's serial 0 shows `12:00:00 AM`
  (pivot-export/t4177-cache-dates-1904-refresh P!A4; T-4943 owns implementing it).
Write one probe case per the `excel-probe` skill (refreshOnLoad pivots, sidecar `text` read back), fill it with
excel-graph, and record the findings. Code that would change: `date_item_format`
(crates/rustsheet-engine/src/pivot/layout/render.rs).

## Acceptance criteria
- [ ] A filled probe case covering the five edges above, with each finding (or its absence) in pivots.md.
- [ ] Failing cells owned by tasks (new-case owners gate).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the rules found (file engine tasks).
