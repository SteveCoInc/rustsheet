---
id: T-5050
title: "Save a label-sorted Top-N row field's rowItems as Excel does"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine's export of a pivot whose row field is label-sorted descending under a Top-N filter saves the `rowItems` Excel saves for it, so the pivot-export suite's layout ratchet holds no item for it.

## Context
Found by T-4944, which made an engine-export case's pivot layout a baseline item (`LF` lines of eval/baseline/pivot-export.txt). pivot-export/t4177-spike.export.json is the export of probes/t4128-pivot-spike; its pivot K!CalcTopSort (K!A3:D11: rows Region then Product, Product `sortType="descending"` with a Top-2 count filter, a calculated field Double) renders every cell as Excel does, and its colItems equal Excel's, but 2 of its 7 rowItems differ (excel-graph sidecar, 2026-11-07):
- Excel: `x=[0,1]`, `r=1 x=[2]`, default `x=[0]`, `x=[1,1]`, `r=1 x=[2]`, default `x=[1]`, grand;
- engine: the same, except `r=1 x=[0]` in both second rows (the item A under B).
Product B is index 1 in both, A is 2 in Excel and 0 in the engine. A reading consistent with that, unverified: Excel saves the field's pivotField `<items>` in its descending label order (C, B, A) and indexes into that, while the export keeps cache order (A, B, C). The spike export's sidecar predates the current export (its `input_sha256` no longer matches; `show` warns), so first refill it (`rustsheet-oracle fill --force` on that case) and confirm the rowItems still differ. Check the sidecar's saved pivotTable part (`rustsheet-oracle` keeps Excel's file) before implementing, and record the finding in docs/excel-semantics/pivots.md. Code: the export's pivotField items and rowItems (crates/rustsheet-xlsx/src/export/pivots/).

Owner of this pivot layout (decision 0245; T-4944):
- pivot-export/t4177-spike.export.json K!CalcTopSort (K!A3:D11)

## Acceptance criteria
- [ ] pivots.md records the order of a label-sorted field's saved `<items>` and the `x` its rowItems use, with Excel evidence.
- [ ] The spike export case has no `LF` line for K!CalcTopSort (K!A3), and the probe it exports passes every cell it passes now.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
