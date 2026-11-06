# Pivot tables (M16)

Rules for rendering, refreshing and authoring pivot tables. GETPIVOTDATA rules and the import limits
stay in web.md (SPEC-0015). Program: docs/research/pivot-tables-2026-11-04/plan.md, decision 0274.

Entries marked UNVERIFIED are observations from throwaway excel-graph runs (the 2026-11-04 spike in
docs/research/pivot-tables-2026-11-04/oracle-spike.md, and the case-builder-v2 runs of T-4122). Each
names the probe task that commits its evidence. Do not implement one before that. VERIFIED entries cite
committed probes.

## Excel for the web re-renders a refreshOnLoad pivot when it opens the file
Rule: a pivot whose cache has `refreshOnLoad="1"` is rebuilt from its source and re-rendered on open,
before any calculate or refresh call. The values the file's cache held do not survive.
Evidence: probes/t4128-pivot-spike R (excel-graph, 2026-11-06, T-4128). R's area held no cells, so
whether Excel also overwrites rendered constants already in the area is a separate, open question
(T-2327). DataF!C2 is a formula
(`=1000`). The sheet caches it as 0 and the pivot cache record is missing (`<m/>`), yet R!B5 (East/A) is 1004, R!E5 is 1012 and R!E7 is 1039.
The saved file holds Excel's own cache and definition (`rowItems`, `location`), as oracle-spike.md
run 1 also showed.
Status: VERIFIED (excel-graph)

## Excel for the web renders authored pivot definitions
Rule: a definition written into the file renders in full on open, with no `rowItems`/`colItems` and a
`location` of any size. Excel computes the layout, the values, the group items and the location. It
needs:
- a cache consistent with itself (shared items, records, flags);
- `<items>` on every axis field;
- an area holding no foreign cells.

Without items or records Excel refuses the file (`OpenWorkbookBlockedWorkbook`, oracle-spike.md run
2, a scratch observation).
Evidence: every pivot of probes/t4123-pivot-capture and probes/t4128-pivot-spike except S, the
`#SPILL!` control (excel-graph, 2026-11-06): compact, tabular, a calculated field with a Top-N filter and a sort, numeric grouping.
The case builder relies on it (tools/src/rustsheet_tools/pivot_definition.py).
Status: VERIFIED (excel-graph) for rendering; the refusal of an inconsistent package is UNVERIFIED
(probe pending: T-4134)

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
Evidence: probes/t4128-pivot-spike K!D6 28.1, K!D7 38.1 (East Total, 19*2+0.1) and K!D11 92.1 (Grand
Total, 46*2+0.1) (excel-graph, 2026-11-06, T-4128).
Status: VERIFIED (excel-graph); edge cases (errors, Count pivots, non-value references) in T-4505

## A Top-N filter changes the subtotals and grand totals
Rule: with a `top10 val="2"` count filter on Product by Sum of Sales, each region shows only its two
largest products. The subtotals and the grand total cover only the products shown: East Total is 19,
not 22 (C dropped), and the Grand Total is 46, not 49. A `sortType="descending"` label sort on Product
lists B before A within each region.
Evidence: probes/t4128-pivot-spike K!A5:C11 (excel-graph, 2026-11-06, T-4128): East B 5, A 14, Total
19; West B 7, A 20, Total 27; Grand Total 46.
Status: VERIFIED (excel-graph); ties, percent and sum Top-N and filter order are in T-4164

## Numeric and date group labels are rewritten by Excel
Rule: verified and recorded by the M6 grouping work. See web.md "Grouped pivot fields import as their
group items" (T-4225, T-4228, T-4235, T-4236, T-4249, T-4253, T-4287, T-4311, T-4316). It covers
record placement, Excel's own labels whatever the file lists, whole-number labels `lo-(lo+interval-1)`,
relabel order, and groupings dropped over blanks or dates. probes/t4128-pivot-spike G (`45000-45100`,
`45100-45200`, `45300-45400` for fractional data, empty groups hidden) agrees with it.
Status: VERIFIED (excel-graph; evidence in web.md)

## A pivot that would meet a foreign cell collapses to #SPILL!
Rule: when a pivot's layout would overwrite cells it does not own, Excel for the web renders nothing
but `#SPILL!` in the anchor cell, and saves the pivot's `location` as that one cell. This matches
Microsoft's 2025 "PivotTable #SPILL!" announcement (excel-docs.md, PSPILL).
Evidence: probes/t4128-pivot-spike S (excel-graph, 2026-11-06, T-4128): the constant S!C5 lies inside
the layout, and the pivot is saved with location A3, where S!A3 is `#SPILL!`.
Status: VERIFIED (excel-graph); GETPIVOTDATA against such a pivot is in T-4130

## A tabular pivot's first row header is `Row Labels` unless the data field is not compact
Rule: in a tabular pivot (`outline="0"`, `compact="0"` on the table and on both row fields), the header
cell of the first row field reads `Row Labels`. It reads the field's own name (`Region`) when the data
field's `pivotField` (Sales, which is on no axis) carries `compact="0"`; its `outline` makes no
difference. The second row field's header is its name (`Product`) in every variant.
Evidence: probes/t4123-pivot-capture A4 (excel-graph, 2026-11-06, T-4123). The four tabular sheets
differ only in the Sales field:
- T (no attributes): `Row Labels`;
- T2 (`compact="0" outline="0"`): `Region`;
- T3 (`compact="0"` only): `Region`;
- T4 (`outline="0"` only): `Row Labels`.

oracle-spike.md run 4, whose uncommitted hand-written XML set `compact="0"` on that field, showed
`Region` too, which this rule likely explains. Probed with one source field, Sales, behind every data field of T to T4. Observation, cause not
isolated: probes/t4128-pivot-spike K!A4 is `Row Labels` with Sales `compact="0"` and the calculated
field Double at the default (T-4128). K also has a Top-N filter and a
sort, so whether every data field must be non-compact, and the layout without a Values axis, are left
to T-4137 and T-4138.
Status: VERIFIED (excel-graph)

## Rendered pivot cells: blanks, repeated labels, subtotals and the Values caption
Rule:
- A row and column intersection with no records is a blank cell, not 0 (compact C!D6, West/C).
- In tabular form an outer item's label shows on its first line only; the lines below it leave the
  outer cell blank (T!A6, A7, A10).
- A subtotal line reads `<item> Total` in the outer field's column, with the inner field's cell blank
  (T!A8 `East Total`, B8 blank; A11 `West Total`), and the grand total line reads `Grand Total`
  (T!A12).
- With the Values field on columns, `Values` sits above the first data field's column (T!C3), and the
  cells above the row fields (A3, B3) are blank.
- A compact pivot with a column field puts the data field's caption (`Sum of Sales`) at the
  location's corner and `Column Labels` beside it (C!A3, B3), with `Row Labels` below (C!A4).

Evidence: probes/t4123-pivot-capture, the `pivots` cells of its sidecar (excel-graph, 2026-11-06,
T-4123). Scored once T-4124 lands; the engine renders none of these yet.
Status: VERIFIED (excel-graph)

## Page fields render above the location
Rule: page fields render in the rows above the pivot's location, each as `<field name>` beside its
selected item, or `(All)` when none is selected.
Evidence: T-4122 runs (sheet Pg: `Region | East`, `Product | (All)`, location A5).
Status: UNVERIFIED (probe pending: T-4138)
Capture assumption (T-4123): oracle sidecars capture the page area as the `rowPageCount` rows above
the location, ending one blank row before it, and `3 * colPageCount - 1` columns wide but at least as
wide as the location (`snapshot.py` `pivot_area`). Only the one-page, one-column shape is evidenced
(Calipers officejs/pivot_filter: one page field at A1:B1, body from A3). T-4138 must confirm the
multi-page offset and the `colPageCount > 1` width, and widen the window if Excel differs.
