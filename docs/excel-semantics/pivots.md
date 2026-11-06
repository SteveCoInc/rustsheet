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
Status: VERIFIED (excel-graph) for rendering. The refusal of an inconsistent package is a builder
observation (T-4122 runs). It cannot become a filled probe, because Excel saves nothing for a file it
refuses.

## Case-variant text items make Excel for the web call the file corrupt
Rule: a cache field whose shared items list two texts that differ only in case (`East`, `east`) makes
Excel for the web refuse the workbook (`FileCorruptTryRepair`). With them merged into one item, the
records of both casings count under it on refresh: East/A summed the `East` and `east` rows (14 = 10
+ 4) and showed `East`.
Evidence: T-4122 runs (cases a and a2: corrupt with both items, rendered with one). The merge on refresh
is VERIFIED by probes/t4134-pivot-items I1 (next entry).
Status: the refusal is a builder observation (T-4122 runs), not committable as a probe: Excel saves
nothing for a file it refuses

## Item identity: exact for numbers and types, case-insensitive for text
Rule: on refresh, texts that differ only in case are one item, and their records add up. In I1 the item
shows `east`, which is both the first-seen and the listed casing, so which of those wins is open. Numbers are compared exactly, so 0.3 and 0.30000000000000004 are two items. The
text `"1"` and the number 1 are two items as well.
Evidence: probes/t4134-pivot-items (excel-graph, 2026-11-06, T-4134):
- I1: east, East and EAST count 3 under `east`, and west counts 1.
- I4: 0.3 counts 2 and 0.30000000000000004 counts 1.
- I5: the number 1 counts 2 and the text `1` counts 1.

Status: VERIFIED (excel-graph)

## Item casing when cache and source disagree, and date items
Rule: unknown. Which casing a merged text item shows when the cache lists one casing and the source's
first row has another, and whether date items with and without a time part are one item. Also open:
whether new items that only a refresh finds are appended in source order or sorted descending
(previous entry, I3).
Evidence: none yet. probes/t4134-pivot-items I1 cannot separate the two casings, because both are
`east`.
Status: UNVERIFIED (probe pending: T-4582)

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
Status: builder observation (T-4122 runs), not committable as a probe: Excel saves nothing for a
file it refuses. The builder writes these flags (pivot_definition.py `shared_items`), and every
filled pivot probe opens with them.

## Items show in the order the pivot field lists them; new items are appended
Rule: on refresh Excel keeps the order of a pivot field's `<items>`. An item the refresh finds that the
field does not list yet goes after the listed ones, not in ascending order. Whether they follow the
source order or a descending sort is open (T-4582). A listed item that has no records after the
refresh is not rendered, and the items after it keep their indices. How items render:
- a number item renders as a number cell;
- a boolean as the text `TRUE` or `FALSE`;
- an error item as the text of the error (`#N/A`), not an error value;
- a blank as `(blank)`.

Evidence: probes/t4134-pivot-items (excel-graph, 2026-11-06, T-4134):
- I2: the builder lists a, FALSE, 1, b, TRUE, 2, then #N/A and blank (first seen); Excel shows that
  order in I2!A4:A11, with A6 and A9 numbers, A10 the text `#N/A` and A11 `(blank)`.
- I3: Zed and Alpha come from formulas the cache records as missing; Excel shows M, Zed, Alpha. The
  source order is M, Zed, Alpha, M, which also reads as a descending sort of the new items. The listed
  `<m/>` item (index 1) holds no record after the refresh and is left out: the saved `rowItems` are
  x = 0, 2, 3.

The default order of a newly created pivot (ascending, by type) is a UI action and not reachable from a
file.
Status: VERIFIED (excel-graph)

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

## A tabular pivot's first row header is `Row Labels` while a non-axis field is compact
Rule: in a tabular pivot, the header cell of the first row field reads `Row Labels` whenever some
`pivotField` on no axis (a data-only field, an unused source field or a calculated field) is at the
default `compact`. In every probe that showed the field's own name (`Region`), every `pivotField`
carried `compact="0"`. `outline` on the non-axis field makes no difference. The second row field's
header is its name (`Product`) in every variant probed. Open (T-4583): whether the rule is "every
field", "every non-axis field" or "the last field". In every probe the default field was also the
pivot's last field, and no probe left an axis field at the default.
Evidence (excel-graph, 2026-11-06):
- probes/t4123-pivot-capture A4 (T-4123), whose source has three fields, all on an axis or used as data:
  - T, with Sales compact by default: `Row Labels`;
  - T2 and T3, with Sales `compact="0"`, every field non-compact: `Region`;
  - T4, with Sales outline off only: `Row Labels`.
- probes/t4128-pivot-spike K!A4 (T-4128): a calculated field at the default gives `Row Labels`.
- probes/t4137-pivot-captions K6!A4 (T-4137): every axis and data field non-compact, but the unused
  source field `Risky` at the default, gives `Row Labels`. K5!A3 (two unused fields at the default)
  gives `Row Labels` too.

oracle-spike.md run 4, whose uncommitted hand-written XML set `compact="0"` on the Sales field, showed
`Region` too.
Correction: T-4123 first read the rule as "the data field's `compact`". K5 and K6 show that an unused
field counts too.
Status: VERIFIED (excel-graph) as stated; which fields count is UNVERIFIED (probe pending: T-4583)

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

## Caption attributes, empty and error captions, page captions, custom subtotal labels
Rule:
- `rowHeaderCaption` replaces `Row Labels` (K1!A4 `Regions`).
- `grandTotalCaption` replaces `Grand Total` (K1!A7 `All of it`).
- `dataCaption` replaces the `Values` caption over the data fields (K1!B3 `Measures`).
- `colHeaderCaption` (`Products`) shows nowhere in K1, whose column axis holds only Values; whether it
  replaces `Column Labels` beside a real column field is open (T-4583).
- With `showMissing` and `missingCaption`, an empty intersection shows the caption (K2!D6 `n/a`).
- With `showError` and `errorCaption`, under Sum, every value cell whose aggregate is an error shows
  the caption as text: the cell, its row and column totals and the grand total (K3!C5, E5, C7 and E7 `ERR`; the
  data hold one `#DIV/0!` at East/B). The empty West/C stays blank (K3!D6).
- Page fields show `(All)` with no selection, the item with one (`A`), and `(Multiple Items)` when
  several items stay visible (K4!B2:B4).
- Custom subtotal functions label each subtotal line `<item> <Function>`: `East Sum`, `East Count`
  (K5!A7:A8), with Count counting records (4).
- With a single data field there is no Values row: the row header sits at the location's corner and
  the data field's caption beside it (K5!A3 `Row Labels`, C3 `Sum of Sales`).

Evidence: probes/t4137-pivot-captions (excel-graph, 2026-11-06, T-4137). Default data-field names
(`Sum of`, `Count of`...) are written by the file, not chosen by Excel, so a file cannot probe them;
creating a pivot in the UI is where Excel picks them (T-2327 keeps that question). Other error kinds
and aggregates under `errorCaption` are not probed.
Status: VERIFIED (excel-graph)

## Page fields render above the location
Rule: page fields render in the rows above the pivot's location, each as `<field name>` beside its
selected item, or `(All)` when none is selected (`(Multiple Items)` with several: previous entry).
Evidence: T-4122 runs (sheet Pg: `Region | East`, `Product | (All)`, location A5).
Status: UNVERIFIED (probe pending: T-4138)
Capture assumption (T-4123): oracle sidecars capture the page area as the `rowPageCount` rows above
the location, ending one blank row before it, and `3 * colPageCount - 1` columns wide but at least as
wide as the location (`snapshot.py` `pivot_area`). probes/t4137-pivot-captions K4 is consistent with it
(location A6, three page fields in A2:B4, blank row 5; T-4137). A capture cannot confirm its own
window, though: row 1 was never read. T-4138 must confirm the window and the `colPageCount > 1`
width.
