# Pivot tables (M16)

Rules for rendering, refreshing and authoring pivot tables. GETPIVOTDATA rules and the import limits
stay in web.md (SPEC-0015). Program: docs/research/pivot-tables-2026-11-04/plan.md, decision 0274.

Until probes land, every entry below is an **observation** from throwaway excel-graph runs on
2026-11-04: the spike (docs/research/pivot-tables-2026-11-04/oracle-spike.md) and the case-builder-v2
runs of T-4122. Each is UNVERIFIED and names the probe task that commits its evidence. Do not implement
any of them before that.

## Excel for the web re-renders a refreshOnLoad pivot when it opens the file
Rule: a pivot whose cache has `refreshOnLoad="1"` is rebuilt from its source and re-rendered on open,
before any calculate or refresh call. The saved file then holds Excel's cache (`refreshedBy="Excel
Services"`), its pivot definition (`rowItems`/`colItems`, `location`) and its rendered cells. Stale
cells and records the file held are replaced.
Evidence: oracle-spike.md run 1 (Data!C2 changed from 10 to 1000 gave East/A 1004, grand total 1039).
Status: UNVERIFIED (probe pending: T-4128; it closes T-2327)

## Excel for the web renders authored pivot definitions
Rule: a definition written into the file renders in full on open, with no `rowItems`/`colItems` and a
`location` of any size. Excel computes the layout, the values, the group items and the location. It
needs:
- a cache consistent with itself (shared items, records, flags);
- `<items>` on every axis field;
- an area holding no foreign cells.

Without items or records Excel refuses the file (`OpenWorkbookBlockedWorkbook`).
Evidence: oracle-spike.md runs 2, 4–6; T-4122 runs (tabular layout with three data fields,
percentOfTotal, a calculated field with Top-N and a descending sort, numeric groups, two page fields
with a hidden item).
Status: UNVERIFIED (probe pending: T-4128). The case builder relies on it
(tools/src/rustsheet_tools/pivot_definition.py).

## Case-variant text items make Excel for the web call the file corrupt
Rule: a cache field whose shared items list two texts that differ only in case (`East`, `east`) makes
Excel for the web refuse the workbook (`FileCorruptTryRepair`). With them merged into one item, the
records of both casings count under it on refresh: East/A summed the `East` and `east` rows (14 = 10
+ 4) and showed `East`.
Evidence: T-4122 runs (cases a and a2: corrupt with both items, rendered with one).
Status: UNVERIFIED (probe pending: T-4134, which also asks which casing wins)

## Shared-item flags Excel accepts for mixed fields
Rule: Excel for the web refuses a cache whose `sharedItems` flags contradict its items. The flags that
open are the ones Excel itself writes:
- booleans and errors count as strings;
- `containsSemiMixedTypes="0"` only when the field has no string and no blank;
- `containsString="0"` when it has no string;
- `containsMixedTypes="1"` when strings sit beside numbers;
- numbers with blanks carry `containsBlank="1"` and no `containsSemiMixedTypes="0"`.

Evidence: T-4122 runs (bool+text, error+text and number+blank refused with other flags, rendered with
these); Excel-saved caches in the LibreOffice fixtures (oss-engines.md §5).
Status: UNVERIFIED (probe pending: T-4134)

## Items show in the order the pivot field lists them
Rule: on refresh Excel keeps the order of a pivot field's `<items>` for existing items; it does not
re-sort them. A field listing `a`, blank, `b` shows `a`, `(blank)`, `b`. Booleans show as `TRUE` and
`FALSE`, errors as their text (`#N/A`), and blanks as `(blank)`.
Evidence: T-4122 runs (f_blankstr, f_boolstr, f_err, f_num_blank).
Status: UNVERIFIED (probe pending: T-4134, together with the default order of a newly created pivot)

## A calculated field applies its formula to the field sums
Rule: a calculated field `Sales*2+0.1` shows, at every level including subtotals and the grand total,
the formula over the sum of Sales, not the sum of the formula over the records. East/A (Sales 10 and
4) shows 28.1, not 28.2.
Evidence: oracle-spike.md run 5; T-4122 runs (sheet C).
Status: UNVERIFIED (probe pending: T-4128, then T-4505 for the edge cases)

## A Top-N filter changes the subtotals and grand totals
Rule: with a `top10 val="2"` count filter on Product by Sum of Sales, the subtotals and grand totals
cover only the products shown. East Total is 19, not 22 (C is dropped).
Evidence: oracle-spike.md run 5; T-4122 runs (sheet C).
Status: UNVERIFIED (probe pending: T-4128)

## Numeric and date group labels are rewritten by Excel
Rule: verified and recorded by the M6 grouping work. See web.md "Grouped pivot fields import as their
group items" (T-4225, T-4228, T-4235, T-4236, T-4249, T-4253, T-4287, T-4311, T-4316). It covers
record placement, Excel's own labels whatever the file lists, whole-number labels `lo-(lo+interval-1)`,
relabel order, and groupings dropped over blanks or dates. The spike's run 6 (`45000-45100` for
fractional data) agrees with it.
Status: VERIFIED (excel-graph; evidence in web.md)

## A pivot that would meet a foreign cell collapses to #SPILL!
Rule: when a pivot's new layout would overwrite cells it does not own, Excel for the web shows
`#SPILL!` in the anchor cell and renders nothing else. This matches Microsoft's 2025 "PivotTable
#SPILL!" announcement (excel-docs.md, PSPILL).
Evidence: oracle-spike.md run 3.
Status: UNVERIFIED (probe pending: T-4128)

## Page fields render above the location
Rule: page fields render in the rows above the pivot's location, each as `<field name>` beside its
selected item, or `(All)` when none is selected.
Evidence: T-4122 runs (sheet Pg: `Region | East`, `Product | (All)`, location A5).
Status: UNVERIFIED (probe pending: T-4138)
