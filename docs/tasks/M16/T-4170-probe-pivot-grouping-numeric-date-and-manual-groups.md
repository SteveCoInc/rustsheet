---
id: T-4170
title: "Probe the pivot grouping cases M6 left open: manual groups, nested date groups, shared groups"
milestone: M16
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4130]
---

## Goal
Evidence for the grouping shapes the M6 grouping work (T-4225..T-4324) did not probe: manual text groups (`<discretePr>`, `Group1` labels, ungrouped items beside them), several date levels on one field (Years + Quarters + Months together) and their layout, a group shared by two pivots of one cache, and grouping inside a tabular or outline layout with subtotals.

## Context
Rescoped 2026-11-06. M6 settled numeric range groups (record placement, labels, relabel order, GETPIVOTDATA number items) and single-level date groups, recorded in web.md "Grouped pivot fields import as their group items" with probes t4228-*, t4235-*, t4236-*, t4249-*, t4253-*, t4287-*, t4311-*, t4316-*; do not re-probe those. Use case builder v2 (`definition`, T-4122), extending it for `<discretePr>` and multi-level date groups. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Probe cases with sidecars for each shape; entries in docs/excel-semantics/pivots.md (or web.md beside the M6 entry).
- [x] Builder v2 writes manual and multi-level date groups (unit tests).
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
- Numeric and single-level date groups (done in M6).

## Outcome
Split to stay size M: manual groups and date levels, in compact, outline and tabular forms with default and custom
subtotals (which covers "grouping inside a tabular or outline layout with subtotals"), are done; a group shared by two
pivots of one cache needs the builder to put two pivots on one cache, so it moved to T-4802 with the other shapes
listed UNVERIFIED below.

Builder v2 (tools/src/rustsheet_tools/pivot_groups.py; Rust mirror crates/rustsheet-eval/src/pivot_definition/groups.rs,
with the pivotField writer moved to pivot_definition/pivot_field.rs for the file-size limit): a `discrete` field entry
adds a `databaseField="0"` group field (`<fieldGroup base>`, `<discretePr>`, `<groupItems>` ungrouped items then
groups) and gives the source field `<fieldGroup par>`; a `date_group` entry groups the field in place by its first
level and adds Excel-named fields (`Quarters`, `Years`, ...) for the others, with Excel's labels and `par` on the last
added field, as Excel-authored caches show (ClosedXML shared-group-field and shared-nested-dategroup, corpus-bench
52379 and 59932). Same refusals on both sides (keys, names, items, bounds of 256, level order, serial range,
combinations with calculated fields and items, data and page uses, a bound past the double range), and an added
field takes T-4164's `item_order` (one helper, `_item_order` / `item_order.rs`, now continuing in the group's own order); texts fold with `str.lower` in Python as `to_lowercase` in Rust
(`fold`, so `ß` and `ss` stay apart on both sides; it was `casefold`, which no committed probe input depends on: every
builder-v2 probe input with a sidecar rebuilds byte-identical). Tests in tools/tests/test_pivot_groups.py and
groups_tests.rs, goldens t4170-pivot-manual-groups__MS and t4170-pivot-date-levels__DS. `iso_date`,
`date_shared_items` and `DATE_GROUPS` moved to pivot_xml.py, shared with pivot.py (same bytes).

Probes (excel-graph, 2026-11-07): probes/t4170-pivot-manual-groups (MC, MO, MT, MS, MG, MB, MP; Calc!B1:B19) and
probes/t4170-pivot-date-levels (DC, DO, DT, DS, DY, DM, DX; Calc!B1:B21). Findings in pivots.md "Manual groups and
date levels as Excel for the web renders them" and web.md "GETPIVOTDATA on manual groups and date levels (T-4170)":
the web opens both shapes; group labels are the file's text (no renumbering, a renamed group and a group of one
shown), in pivotField order (MP's North, F, Group1, Group2 rules out the groupItems order, groups-then-ungrouped,
first-source-item order and sorting); an ungrouped item is its own group item; date levels nest (January splits by year,
merges with months alone), `<start` and `>end` items appear at every level; tabular `Total`, custom `Sum`/`Count`
lines per group and level; GETPIVOTDATA on labels, pairs and triples, `#REF!` for an inner label under several outer
items and for a field with custom subtotals only. No shape was refused.

Engine: 35 of 215 and 155 of 431 checked cells pass. Owners (decision 0245): T-4171 for both causes (the importer
skips a cache with a `par`-only `<fieldGroup>`; the load-time refresh loses the added date fields), T-4154 for the
nested-axis GETPIVOTDATA cells of DC.
