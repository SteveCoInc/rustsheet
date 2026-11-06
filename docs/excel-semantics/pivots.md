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
shows `east`; see "Item casing when cache and source disagree, and date items" (T-4582). Numbers are
compared exactly,
so 0.3 and 0.30000000000000004 are two items. The
text `"1"` and the number 1 are two items as well.
Evidence: probes/t4134-pivot-items (excel-graph, 2026-11-06, T-4134):
- I1: east, East and EAST count 3 under `east`, and west counts 1.
- I4: 0.3 counts 2 and 0.30000000000000004 counts 1.
- I5: the number 1 counts 2 and the text `1` counts 1.

Status: VERIFIED (excel-graph)

## Item casing when cache and source disagree, and date items
Rule (as observed):
- **Casing.** When the cache lists one casing (`EAST`) and the source holds several, Excel keeps the
  listed item's slot but relabels it. In J1 the label became `east`, which is the first source
  record's casing, and also the all-lowercase and the ordinally greatest one. Which of those decides
  is open (T-4612). The listed casing does not win.
- **Date items.** A date-formatted source value does not match a cached number item of the same
  value. Its records leave the listed `<n>` items empty, so those are not rendered, and Excel appends
  new items for them. A date with a time part is its own item: 45000 and 45000.5 are two items under
  both formats probed. The appended items render as their serials, as number cells. Their displayed
  format is not captured by the sidecar (displayed text, T-4169).

  Plain numbers do reuse the listed slots (probes/t4134-pivot-items I4, `rowItems` x = 0, 1). Whether
  the appended items are date-typed (`<d>`) is open (T-4612). The engine's refresh (T-4141) must not
  reuse a number item for a date value.

Evidence: probes/t4582-pivot-item-edges (excel-graph, 2026-11-06, T-4582):
- J1: the source's K is the formula `="east"`, then `EAST`, `East` and `west`. The builder lists
  `<m/>`, `EAST`, `west`. The saved `rowItems` are x = 1 (shown `east`, count 3; J1!A4:B4) and x = 2
  (`west`, 1). So the `EAST` slot is reused and relabeled, and the `<m/>` slot is dropped, as in J2
  and t4134 I3.
- J3 (`m/d/yyyy`) and J4 (`m/d/yyyy h:mm`): K is 45000, 45000.5, 45000, 45001. The builder lists
  `<n>` items 45000, 45000.5 and 45001. The saved `rowItems` are x = 3, 4, 5: three appended items,
  shown 45000 (count 2), 45000.5 (1) and 45001 (1) as number cells (J3!A4:A6, J4!A4:A6). The listed
  items 0–2 are not rendered.

Status: VERIFIED (excel-graph) for the cells and items above. The casing rule's deciding property and
the appended items' type are UNVERIFIED (probe pending: T-4612).

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
field does not list yet goes after the listed ones, in source order (first appearance), not sorted.
A listed item that has no records after the
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
- probes/t4582-pivot-item-edges J2 (T-4582): new items in the non-monotone source order Alpha, Zed,
  Beta (all formulas the cache records as missing) after the listed M. Excel shows M, Alpha, Zed, Beta
  (J2!A4:A7), which is source order, not ascending (Alpha, Beta, Zed) and not descending.

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

## A tabular pivot's first row header is the field name only when every pivotField has compact="0"
Rule: in a tabular pivot, the header cell of the first row field reads the field's own name (`Region`)
only when every `pivotField` carries `compact="0"`. Otherwise it reads `Row Labels`. One field at the
default gives `Row Labels` in every role and position probed:
- roles: the outer row field, the inner row field, the data field, an unused source field and a
  calculated field;
- positions in the cache: first (R1), middle (R3, R4) and last (R6, and t4128 K's calculated field).

Column and page fields were not probed. For the data field, its `outline` makes no difference (t4123
T3, T4); for other fields `outline` was not varied apart from `compact`. The second row field's header
is its name (`Product`) whenever it keeps its own column.

An outer row field left at the default `compact` and `outline` (R1) also changes the layout:
- the inner field moves into the outer field's column: location A3:B11 against A3:C11, and
  firstDataCol 1 against 2;
- there are no `<item> Total` lines; each outer item's subtotal sits on the item's own line (R1!A4
  `East` with B4 22, R1!A8 `West` with B8 27, and West's products in A9:A10), while the other R sheets
  have `East Total` at A7 and `West Total` at A10.

Which of the two attributes causes this is not separated.
Evidence (excel-graph, 2026-11-06):
- probes/t4583-pivot-row-labels A3 (T-4583). The cache fields are Region, Product, Sales, Units and
  Risky; every field carries `compact="0" outline="0"` except the one named:
  - R1 Region (outer axis, first): `Row Labels`;
  - R2 Product (inner axis): `Row Labels`;
  - R3 Units (unused, middle): `Row Labels`;
  - R4 Sales (data, middle): `Row Labels`;
  - R6 Risky (unused, last): `Row Labels`;
  - R5 (none left at the default): `Region`.
- probes/t4123-pivot-capture A4 (T-4123): T and T4, with Sales at the default, give `Row Labels`; T2
  and T3, with every field non-compact, give `Region`.
- probes/t4128-pivot-spike K!A4 (T-4128): a calculated field at the default gives `Row Labels`.
- probes/t4137-pivot-captions K5!A3 and K6!A4 (T-4137): unused fields at the default give `Row Labels`.

oracle-spike.md run 4, whose uncommitted hand-written XML set `compact="0"` on the Sales field, showed
`Region` too.
Correction: T-4123 first read the rule as "the data field's `compact`". T-4137 narrowed it to the
non-axis fields, and T-4583's R1 and R2 show that axis fields count too.
Status: VERIFIED (excel-graph) for the roles and positions probed. UNVERIFIED (probe pending: T-4609):
column and page fields, `outline` alone on axis fields, and which attribute moves R1's subtotals.

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
- `colHeaderCaption` replaces `Column Labels` beside a real column field in compact form
  (probes/t4583-pivot-row-labels C1!B3 `Products`, with rowHeaderCaption C1!A4 `Regions`). In K1,
  whose column axis holds only Values, it shows nowhere. In tabular form with every field non-compact
  it shows nowhere either: the column header reads the field's name (C2!B3 `Product`) and the row
  header reads its name too (C2!A4 `Region`; C2 sets no rowHeaderCaption).
- With `showMissing` and `missingCaption`, an empty intersection shows the caption (K2!D6 `n/a`).
- With `showError` and `errorCaption`, under Sum, every value cell whose aggregate is an error shows
  the caption as text: the cell, its row and column totals and the grand total (K3!C5, E5, C7 and E7 `ERR`; the
  data hold one `#DIV/0!` at East/B). The empty West/C stays blank (K3!D6).
- Page fields show `(All)` with no selection, the item with one (`A`), and `(Multiple Items)` when
  several items stay visible (K4!B2:B4).
- Custom subtotal functions label each subtotal line `<item> <Function>`: `East Sum`, `East Count`
  (K5!A7:A8), with Count counting records (4).
- With a single data field and no column field (tabular, K5) there is no Values row. `Row Labels` is
  at the location's corner (K5!A3), the inner row field's name beside it (B3 `Product`), and the data
  field's caption above the data column (C3 `Sum of Sales`). With a column field the corner holds the
  data caption instead (K2!A3, K3!A3; previous entry).

Evidence: probes/t4137-pivot-captions (excel-graph, 2026-11-06, T-4137). Default data-field names
(`Sum of`, `Count of`...) are written by the file, not chosen by Excel, so a file cannot probe them;
creating a pivot in the UI is where Excel picks them (T-2327 keeps that question). For other
aggregates and `#DIV/0!` under `errorCaption`, see "errorCaption replaces every error value" below.
Status: VERIFIED (excel-graph)

## Page fields render above the location
Rule: page fields render in the rows above the pivot's `location`, ending one blank row before it. Each
shows `<field name>` beside its selection: `(All)`, the item, or `(Multiple Items)`. With
`pageWrap="n"` and `pageOverThenDown`, the fields fill a row n at a time, each field-item pair two
columns wide with a blank column between pairs. The saved `rowPageCount` and `colPageCount` give the
grid (here 2 by 2).
Evidence:
- probes/t4137-pivot-captions K4 (T-4137): three page fields in A2:B4 above location A6.
- probes/t4138-pivot-geometry G8 (T-4138): three page fields wrapped two per row, `Region | (All)` in
  B5:C5, `Product | (All)` in E5:F5 and `Channel | (All)` in B6:C6, row 7 blank, location B8,
  `rowPageCount=2`, `colPageCount=2`.

Capture window (T-4123, `snapshot.py` `pivot_area`): the `rowPageCount` rows above the location, minus
one blank row, and `3 * colPageCount - 1` columns wide. Both shapes above are consistent with it. In G8
the second pair starts at E, three columns after B, and the five-column window ends at F. A capture
cannot see past its own window, though, so a page cell outside it would go unseen (row 1 of K4,
rows 1–4 of G8).
Status: VERIFIED (excel-graph) for the rendering; the window is consistent with both probes, not
independently confirmed (probe pending: T-4584)

## Pivot geometry: location and header offsets per layout
Rule: Excel writes the pivot's `location` as the body's bounding box (page fields excluded), with
`firstHeaderRow`, `firstDataRow` and `firstDataCol` measured from its top left. With the location's top
left at B4 (probes/t4138-pivot-geometry, T-4138):

| sheet | layout | location | fhr, fdr, fdc | rowItems, colItems |
|---|---|---|---|---|
| G1 | compact, 1 row field | B4:C7 | 1, 1, 1 | 3, 1 |
| G2 | compact, 2 row fields, 1 column field | B4:E13 | 1, 2, 1 | 8, 3 |
| G3 | outline, 2 row fields, 2 data fields | B4:E13 | 1, 2, 2 | 8, 2 |
| G4 | tabular, 3 row fields | B4:E18 | 1, 1, 3 | 14, 1 |
| G5 | compact, 1 row field, 2 column fields | B4:J9 | 1, 3, 1 | 3, 8 |
| G6 | compact, Values on rows | B4:C12 | 1, 1, 1 | 8, 1 |
| G7 | 1 column field only | B4:F6 | 1, 2, 1 | 1, 4 |
| G9 | 3 data fields, no axis field | B4:D6 | 1, 2, 0 | 1, 3 |

In compact form all row fields share one column (fdc 1). Outline and tabular give each row field its own
column (fdc = number of row fields), but each rests on one probe (G3, G4). With no row field, fdc is 1
when there is a column field (G7) and 0 when there is no axis field at all (G9, and G8 in the page-field entry above). fdr is 1
plus the number of `colFields`, and the Values field counts when it is on columns: G3 and G9 (Values
only) have fdr 2, as does G2 (one column field and one data field), and G5 has 3. With Values on rows,
each item is followed by one line per data field, and the grand total reads `Total <data field>`
(G6!B11:B12).

A Values field on rows needs `dataOnRows="1"` on the definition: without it Excel for the web refuses
the file as corrupt (`FileCorruptTryRepair`). Case builder v2 now writes it. This comes from scratch
excel-graph runs of T-4138, not committed: G6's package was refused without the attribute and opened
with it, all else equal.
Status: VERIFIED (excel-graph) for the table; the refusal is a builder observation

## Aggregates over text, blanks, booleans and errors
Rule (each aggregate in a value field that also holds other kinds of value):
- **Skipped values.** Text, blanks and booleans are skipped by Sum, Average, Max, Min, Product, StdDev,
  StdDevp, Var, Varp and CountNums. A boolean is not a number: `TRUE` is not 1.
- **Count** counts every non-blank value: numbers, text, booleans and errors. Its grand total can still
  be an error (see "A Count total over an error field is the error when the pivot mixes
  aggregates").
- **CountNums** counts numbers only. An error does not poison its item cells. Its totals can be the
  error (see "A Count total over an error field is the error when the pivot mixes aggregates").
- **Errors.** An error poisons every aggregate except Count and CountNums. The aggregate shows the error
  (`#N/A`) in the cell holding it and in every subtotal and grand total that covers that cell. Cells that
  do not cover it are unaffected.
- **One value.** Over a single number, StdDev and Var are `#DIV/0!`, while StdDevp and Varp are 0. This
  holds wherever the other values in the cell are text, blank or boolean. A data error wins over it:
  East/A of Err holds one number beside `#N/A` and shows `#N/A` (A_stdDev!L6, A_var!L6).
- **Empty intersections.** An intersection with no records is blank for every aggregate, Count and
  CountNums included.

Evidence: probes/t4135-pivot-aggregates (excel-graph, 2026-11-06, T-4135). Each sheet A_<aggregate> has
rows Region, columns Kind and six value fields. The Data!A2:H5 records are East/B, East/A, East/A and
West/A, and East/A row 4 holds the odd value: text `abc`, a blank, `TRUE` or `#N/A`.
- **Skipped values:** A_sum!I6:K6 are 10 for Txt, Blk and Bool, against H6 = 14 for Num. A_average!I6:K6
  are 10 and A_product!I6:K6 are 10.
- **Count:** A_count!I6, K6 and L6 are 2 for Txt, Bool and Err; J6 is 1 for Blk.
- **CountNums:** A_countNums!I6:L6 are 1.
- **Errors:** column L (Err) of A_sum, A_average, A_max, A_min, A_product, A_stdDev, A_stdDevp, A_var and
  A_varp is `#N/A` at row 6, at the East total R6 and at the grand totals L8 and R8. West/A (L7) shows
  its own one-value result: 7 for Sum to Product, `#DIV/0!` for StdDev and Var, and 0 for StdDevp and
  Varp.
- **One value:** A_stdDev and A_var are `#DIV/0!` at East/B (B6:G6) and West/A (H7:M7), and at I6
  (East/A Txt: 10 and `abc`). A_stdDevp and A_varp are 0 there.
- **Empty intersections:** West/B (B7:G7) is blank on every sheet.
Status: VERIFIED (excel-graph)

## Summation order of Sum, Average and Product totals
Rule:
- **Totals over one axis field.** A Sum, Average or Product total over the items of a pivot's only
  axis field accumulates the records item by item, in the field's display (first-seen) item order,
  and in record order within each item. Average then divides by the count. This is neither record
  order, nor reverse record order, nor ascending item order, nor the sum of the children. For Product
  only one cell separates the orders (P2 B6), and ascending value order fits it too (T-4616).
- **Sum across a column field.** In t4135, a row's Sum total across the column field Kind does not
  follow display-order accumulation, which gives 0.6 there. Three readings fit A_sum!S6 and S8:
  - the sum of the row's cells;
  - ascending column-item order;
  - reverse record order.

  In the same pivot, Average does follow display order: A_sum!S8 is 1.3, while A_average!S8 is
  0.32499999999999996 = 1.2999999999999998 / 4. Which reading holds, and why Sum and Average
  differ, is open (T-4616).

Evidence (excel-graph, 2026-11-06):
- probes/t4614-pivot-total-order-2 (T-4614). D1 holds the records Z 6.5, A 0.1, Z 2.3, A 8.9, Z 2.4,
  A 9.4, with Z first seen. The candidate sums are:
  - record order 29.599999999999994;
  - reverse 29.600000000000005;
  - ascending item order (A, then Z) 29.599999999999998;
  - display item order (Z, then A) 29.6.
- The cells:
  - S2 (rows K): Grand Total Sum 29.6 (B7), and Average 4.933333333333334 = 29.6 / 6 (C7).
  - S3 (K on columns): Grand Total 29.6 (D5).
  - P2 (D2, Product): Grand Total 15743.706264 (B6). Display item order gives that; record order gives
    15743.706264000002.
- probes/t4589-pivot-total-order S1 (T-4589): records A 0.2, B 0.7, A 3.3, B 1.1, with A shown first.
  Its Sum Grand Total is 5.300000000000001 (B7), the display-item-order sum. The children's sum would
  be 5.3, and record order 5.299999999999999.
- probes/t4135-pivot-aggregates (T-4135), field Dec: East records 0.3 (B), 0.2 (A) and 0.1 (A), with
  Kind on columns shown B, then A.
  - A_sum!S6, East's total, is 0.6000000000000001. That is the sum of East's cells
    (0.3 + 0.30000000000000004); item order would give 0.6.
  - A_average!S6 is 0.19999999999999998 = 0.6 / 3, in item order.

Correction: T-4135 first recorded Sum as the sum of the children, and T-4589 left two candidates. The
T-4614 probes settle totals over one axis field.
Status: VERIFIED (excel-graph) for totals over one axis field. UNVERIFIED (probe pending: T-4616): the
order of a Sum total across a column field.

## errorCaption replaces every error value
Rule: with `showError` and `errorCaption`, every error value a pivot cell would show becomes the
caption, as text. That includes `#DIV/0!` from StdDev and Var over one value, not only errors from the
data.
Evidence: probes/t4135-pivot-aggregates EC (excel-graph, 2026-11-06, T-4135):
- East (B5:L5) shows `ERR` for every aggregate except Count (3) and CountNums (2).
- West (B6:L6) shows `ERR` for StdDev and Var, which have one value, and 0 for StdDevp and Varp.
- The Grand Total row (B7:L7) is `ERR` everywhere; see the next entry for Count and CountNums.
Status: VERIFIED (excel-graph)

## A Count total over an error field is the error when the pivot mixes aggregates
Rule: Count and CountNums never show an error in an item's cell. When the pivot's data fields mix
aggregates (Count or CountNums beside Sum or Average), though, every row or column grand total of a
Count or CountNums that covers a record holding an error is that error: `#N/A`, or the errorCaption
when one is set. Grand totals that cover no such record stay counts. With one data field, or only
Count fields, the totals are counts. A column field makes no difference. Two cases are not probed:
inner-field subtotals, and Count beside CountNums (T-4616).
Evidence (excel-graph, 2026-11-06):
- probes/t4614-pivot-total-order-2 (T-4614), over D3 (East A Err 10, East B `#N/A`, West A 7), without
  showError:
  - C1, Count of Err and Count of Num, no column field: Grand Totals 3 and 3 (B7, C7).
  - C2, Count of Err beside Sum of Num, column field Kind:
    - the item cells are counts (B6 1, D6 1, B7 1);
    - East's total, B's grand total and the grand total of Count of Err are `#N/A` (F6, D8, F8);
    - West's total, which covers no error, is 1 (F7).
  - C3, CountNums of Err beside Sum of Num: Grand Total `#N/A` (B7).
  - C4, Count of Err beside Average of Num: Grand Total `#N/A` (B7).
- probes/t4589-pivot-total-order (T-4589):
  - E1, Count alone: Grand Total 3 (B6).
  - E2 and E3, mixed, with errorCaption: `ERR` (B7).
  - E4, mixed, without the caption: `#N/A` (B7).
- probes/t4135-pivot-aggregates (T-4135):
  - A_count, six Count fields: 4 (R8).
  - EC, eleven mixed aggregates: `ERR`.
Status: VERIFIED (excel-graph)

## Pivots without refreshOnLoad, and source edits
Rule: unknown. Two things are unprobed: whether Excel for the web re-renders a pivot whose cache lacks
`refreshOnLoad` when it opens the file (saved cells or `rowItems` stale or absent), and whether it
re-renders a pivot after its source is edited with no refresh.
Evidence: none. Every rendered probe so far is `refreshOnLoad` (case builder v2).
Status: UNVERIFIED (probe pending: T-4586)

## Pivot field names for duplicate and blank source headers
Rule: unknown. How Excel names the cache fields of a source whose header row repeats a name, leaves a
cell blank, or has names differing only in case.
Evidence: none.
Status: UNVERIFIED (probe pending: T-4585)
