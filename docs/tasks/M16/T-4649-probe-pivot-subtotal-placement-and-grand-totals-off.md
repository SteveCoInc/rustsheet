---
id: T-4649
title: "Probe pivot subtotal placement and grand totals off in rowItems and colItems"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md "Layout lines" settles, from Excel's saved `rowItems`/`colItems`, where subtotal and grand-total lines go when the probed defaults are changed.

## Context
T-4147 emits the lines (`crates/rustsheet-engine/src/pivot/layout/lines.rs`). Every committed probe keeps `subtotalTop`, `defaultSubtotal` and both grand totals on, so these readings are the engine's, UNVERIFIED in pivots.md:
- `subtotalTop="0"` on an outline or compact row field: the item's own line keeps no values and a `t="default"` line follows its children;
- `defaultSubtotal="0"` with no custom subtotal: no subtotal line, the item's own line stays (outline) or is dropped (tabular);
- custom subtotals (`sumSubtotal` ...) on an outline or compact row field: lines after the children, never on the item's line;
- `defaultSubtotal="1"` together with custom subtotals: the default line alone;
- `rowGrandTotals="0"` drops the `t="grand"` line of `rowItems` and `colGrandTotals="0"` that of `colItems` (corroborated by an Excel-saved corpus file, oss-engines.md §0.5, not committed);
- a column field: subtotals always after the children, whatever `subtotalTop`.

Variants: compact and outline pivots of two row fields and one column field with each attribute flipped; case builder v2 with excel-graph (the builder manages `defaultSubtotal`, so raw XML or a builder option is needed).

## Acceptance criteria
- [ ] A filled probe with the variants; the sidecars' `row_items`/`col_items` close or correct each reading in pivots.md "Layout lines".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering the cells (T-4149); blank rows and collapsed items (T-4168).
