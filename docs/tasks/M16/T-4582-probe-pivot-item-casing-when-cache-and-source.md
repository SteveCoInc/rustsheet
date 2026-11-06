---
id: T-4582
title: "Probe pivot item casing when cache and source disagree, and date items with and without times"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's rule for three open item questions is evidenced in docs/excel-semantics/pivots.md:
- which casing a pivot item shows when the cache lists one casing and the source's first row has another;
- whether date items with and without a time part (45000 and 45000.5) are one item or two;
- whether items that only a refresh finds are appended in source order or sorted descending. In T-4134 I3 the source order is M, Zed, Alpha, which cannot separate the two; use, say, Alpha, Zed, Beta.

## Context
T-4134 (probes/t4134-pivot-items) verified case-insensitive text identity. But in I1 the first-seen casing and the builder's listed casing are both `east`, so it cannot tell which one wins. Dates were not probed. Build the cases with case builder v2 (tools/src/rustsheet_tools/pivot_definition.py) and fill them with excel-graph. Context: pivots.md "Item identity"; decision 0274.

## Acceptance criteria
- [x] A probe with a listed casing (`EAST`) that differs from the source's first casing (`east`) has a filled sidecar, and pivots.md states which casing renders.
- [x] A probe whose new items arrive in non-monotone source order has a filled sidecar, and pivots.md states the append order.
- [x] A probe with date items 45000 and 45000.5 (date-formatted source) has a filled sidecar, and pivots.md states whether they are one item and how the item renders.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4141 implements cache identity).

## Outcome
probes/t4582-pivot-item-edges (excel-graph, 2026-11-06) holds four builder-v2 pivots. Recorded in
docs/excel-semantics/pivots.md:
- **Casing (J1):** Excel reuses the listed `EAST` slot and relabels it `east`; the listed casing does
  not win. Whether the first record's casing, the lowercase one or the ordinal maximum decides is
  T-4612's.
- **Append order (J2):** items only a refresh finds follow the listed ones in source order: Alpha, Zed,
  Beta, neither ascending nor descending.
- **Dates (J3, J4):** date-formatted values do not match the cached number items. Those stay empty, and
  three new items are appended (`rowItems` x = 3–5). 45000 and 45000.5 are two items under both
  formats probed (`m/d/yyyy` and `m/d/yyyy h:mm`). The appended items render as serial numbers. Their
  type (date items?) is T-4612's, and their displayed format is T-4169's.
