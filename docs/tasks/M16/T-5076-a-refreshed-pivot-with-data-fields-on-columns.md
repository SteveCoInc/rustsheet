---
id: T-5076
title: "A refreshed pivot with data fields on columns writes the Values caption cell above them"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4506]
---

## Goal
A refreshed pivot whose data fields sit on the column axis writes the `Values` caption where Excel's on-open render does.

## Context
Since T-4506, the engine refreshes pivots over a Table. Check (c) (T-4972, decision 0349) finds Excel's on-open render showing `Values` at `pivot-on-open/TemplateWithTableSourcePivotTables.xlsx` StressedAssetsPivot!D4 (PivotTable3, location A4:D8, two page rows) where the refresh writes nothing. `Values` is the en-US data caption (pivot_freshness.rs). The engine's own render may write it in a case the refresh misses; find which path drops it.

Owner of this check (c) cell (decision 0245; T-4972): StressedAssetsPivot!D4, 1 cell.

## Acceptance criteria
- [ ] The refresh writes the caption, and StressedAssetsPivot!D4 passes check (c).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-English captions.
