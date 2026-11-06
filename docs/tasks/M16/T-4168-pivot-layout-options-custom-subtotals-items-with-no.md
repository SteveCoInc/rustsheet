---
id: T-4168
title: "Pivot layout options: custom subtotals, items with no data, repeat labels, blank rows, collapse"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4149]
---

## Goal
The layout honours custom subtotal functions (several per field), show items with no data, repeat item labels, a blank row after each item, per-field form mixes, and collapsed items (`sd="0"`).

## Context
Evidence from cap-layout and the geometry probes; probes added here where missing. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152, whose adapter renders them
  through the engine): the countA and count custom subtotals over a `#N/A` render `#N/A` where Excel
  counts up to the error (pivots.md "Which function the countA and count custom subtotals compute";
  scope T-4636): probes/t4622-pivot-count-subtotals B1!C6:C7, B4!C6:C7, B5!C6:C7, CA!C6:C6,
  CN!C6:C6.

## Acceptance criteria
- [ ] A custom subtotal line's label is Excel's (`<item> Count` for both `t="countA"` and `t="count"`, probes/t4137-pivot-captions K5!A8 and probes/t4622-pivot-count-subtotals B1!A6:A7), never derived from the model's `Aggregate` name (`countA` is Count, `count` CountNums, T-4622). Under a Sum data field an error ends the count within its inner-item cell (pivots.md; scope open, T-4636).
- [ ] Probes for each option pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Styles.
