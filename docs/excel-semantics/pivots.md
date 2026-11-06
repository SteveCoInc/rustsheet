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

## Which function the countA and count custom subtotals compute
Rule: a pivot field's `countASubtotal` (subtotal line `item t="countA"`) is COUNTA: it counts every
non-blank value, text, booleans and errors included (the data-field `count`; but see Errors below). `countSubtotal` (`t="count"`)
counts numbers only (Count Numbers, the data-field `countNums`); text and booleans are not numbers. Both
lines are labelled `<item> Count`, so with both on, two lines read `East Count`; the line's place
(the `<items>` order, `countA` first in B1) is the only visible tell; the file's `rowItems` carry
`t="countA"` and `t="count"`. The subtotal function overrides the data field's
aggregate: under Sum of Val the lines are counts, not sums, and stay numbers while the Sum cells and the
Grand Total show `#N/A`.
Errors, as observed (one outer field with the subtotals, one inner row field, one `#N/A`): under a Sum
data field, the error ends the count within its inner-item cell. In record order, countA counts the error
and nothing after it, and count counts the numbers before it. Other cells count in full. Under a Count
data field the error ends nothing. The scope of this rule is open: T-4636.
Evidence: probes/t4622-pivot-count-subtotals (excel-graph, 2026-11-06, T-4622). Rows Region (custom
subtotals only) and Product, tabular. East holds five records, A: 10, `abc`; B: a blank, `#N/A`, 5. West
holds 7 (A) and `x`, TRUE, 3 (B).
- **countA alone, Sum of Val (CA):** `East Count` 3 (A6, C6), `West Count` 4 (A9, C9).
- **count alone (CN):** `East Count` 1 (C6), `West Count` 2 (C9).
- **Both (B1):** C6:C7 are 3 and 1, C10:C11 4 and 2, and A6, A7, A10 and A11 all read `East Count` or
  `West Count`.
- **Both, no error (B2, Clean, with the `#N/A` replaced by 2):** East is 4 and 3 (C6:C7), West 4 and 2.
  This is COUNTA and COUNT exactly: the blank is skipped, and `TRUE` counts for countA only.
- **Both, Count of Val (B3):** East is 4 and 2 (C6:C7). The error counts for countA and nothing is cut.
- **Error first (B4, East A: `#N/A`, 10; East B: `abc`, a blank, 5):** East is 3 and 1. A counts the
  error only, and B counts in full (2 and 1). Record order across the whole item would give 1 and 0.
- **Error last (B5, East B: a blank, 5, `#N/A`):** East is 4 and 2.
- probes/t4137-pivot-captions K5 (all numbers): `East Count` counts 4.

Status: VERIFIED (excel-graph) for the functions and the labels. The error rule is as observed
(UNVERIFIED beyond the shape probed: T-4636).
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`SUBTOTAL_FUNCTIONS`; the model only,
nothing computes or labels a custom subtotal yet, T-4168)

## Item identity: exact for numbers and types, case-insensitive for text
Rule: on refresh, texts that differ only in case are one item, and their records add up. In I1 the item
shows `east`; see "Item casing when cache and source disagree, and date items" (T-4582, T-4612). Numbers are
compared exactly,
so 0.3 and 0.30000000000000004 are two items. The
text `"1"` and the number 1 are two items as well.
Evidence: probes/t4134-pivot-items (excel-graph, 2026-11-06, T-4134):
- I1: east, East and EAST count 3 under `east`, and west counts 1.
- I4: 0.3 counts 2 and 0.30000000000000004 counts 1.
- I5: the number 1 counts 2 and the text `1` counts 1.

Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`ItemKey::identity`)

## Item casing when cache and source disagree, and date items
Rule:
- **Casing.** When texts that differ only in case merge into one item, the item keeps the listed
  item's slot and takes the casing of the first source record that reaches it, when that record is
  a formula the old cache held as `<m/>` (every probed case). Neither the listed casing nor the
  lowercase one decides. Whether a literal first record, whose value the cache already lists in
  another casing, relabels the slot the same way is UNVERIFIED (T-4654).
- **Date items.** A date-formatted source value does not match a cached number item of the same
  value. Its records leave the listed `<n>` items empty, so those are not rendered, and Excel appends
  new items for them, saved as date items (`<d v="2023-03-15T00:00:00"/>`). A date with a time part is
  its own item: 45000 and 45000.5 are two items under both formats probed. A date and an equal plain
  number in one field are two items as well. A time-only `h:mm` and an elapsed `[h]:mm` format count
  as dates. The appended items render as their serials, as number cells. Their displayed format is
  not captured by the sidecar (displayed text, T-4169).
- **Saved order.** In the saved cache Excel lists the shared items in source order first and the
  stale ones (old items no record holds) after them, while the pivot field's items keep the old order
  with the new ones appended: `rowItems` index the pivot field's order. See T-4654.

  Plain numbers do reuse the listed slots (probes/t4134-pivot-items I4, `rowItems` x = 0, 1).

Evidence: probes/t4582-pivot-item-edges (excel-graph, 2026-11-06, T-4582):
- J1: the source's K is the formula `="east"`, then `EAST`, `East` and `west`. The builder lists
  `<m/>`, `EAST`, `west`. The saved `rowItems` are x = 1 (shown `east`, count 3; J1!A4:B4) and x = 2
  (`west`, 1). So the `EAST` slot is reused and relabeled, and the `<m/>` slot is dropped, as in J2
  and t4134 I3.
- J3 (`m/d/yyyy`) and J4 (`m/d/yyyy h:mm`): K is 45000, 45000.5, 45000, 45001. The builder lists
  `<n>` items 45000, 45000.5 and 45001. The saved `rowItems` are x = 3, 4, 5: three appended items,
  shown 45000 (count 2), 45000.5 (1) and 45001 (1) as number cells (J3!A4:A6, J4!A4:A6). The listed
  items 0–2 are not rendered.

Evidence: probes/t4612-pivot-date-items-and-casing (excel-graph, 2026-11-06, T-4612). The sidecar's
`cache_fields` carry each saved shared item's type (`shared_items`) and the `<sharedItems>`
attributes (`shared_flags`). In C1 to C3 the first record is a formula, which the case builder caches
as `<m/>`, and the "listed" item is the one the builder wrote (pivot_definition.py caches the first
literal's casing), not one the sidecar records:
- C1: K is `="EAST"`, `east`; listed `east`. C1!A4 shows `EAST` (count 2, x = 1): the first record's
  casing over the listed, lowercase and last one.
- C2: K is `="East"`, `EAST`, `east`; listed `EAST`. C2!A4 shows `East` (count 3, x = 1).
- C3: K is `="East"`, `east`, `EAST`; listed `east`. C3!A4 shows `East` (count 3, x = 1). C2 and C3
  rule out the listed, the lowercase and the last record's casing.
- T1 (`m/d/yyyy`): K is 45000, 45001, 45000; listed `<n>` 45000, 45001. Saved `rowItems` x = 2, 3,
  shown 45000 (2) and 45001 (1) (T1!A4:B5). The saved shared items are `<d>` 2023-03-15T00:00:00,
  `<d>` 2023-03-16T00:00:00, then the old `<n>` 45000 and 45001.
- T2 (`h:mm`, 0.25, 0.5, 0.25) and T3 (`[h]:mm`, 1.5, 2.25, 1.5): the same shape. x = 2, 3, shown
  0.25 and 0.5 (T2!A4:A5), 1.5 and 2.25 (T3!A4:A5), saved as `<d>` 1899-12-30T06:00:00 and
  1899-12-30T12:00:00 (T2), 1899-12-31T12:00:00 and 1900-01-01T06:00:00 (T3).
- T4: K is 45000 (`m/d/yyyy`), `x`, 45001 (`m/d/yyyy`); listed `<n>` 45000, `x`, `<n>` 45001. Saved
  x = 1, 3, 4: `x` reuses its slot (T4!A4), the dates are appended (T4!A5:A6). Shared items `<d>`,
  `x`, `<d>`, then the old `<n>` 45000 and 45001.
- T5: K is 45000 (`m/d/yyyy`), then 45000 and 45001 General; listed `<n>` 45000, 45001. Saved
  x = 0, 1, 2, shown 45000, 45001, 45000, each count 1 (T5!A4:B6): the date is a third item. Shared
  items `<d>` 2023-03-15T00:00:00, `<n>` 45000, `<n>` 45001.

Status: VERIFIED (excel-graph), the casing rule for a formula (uncached) first record only; a literal
first record is UNVERIFIED (T-4654). The saved shared-item order and the stale items' place in it are as
observed; what the export must write is T-4654's.
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`FieldReader::push` relabels the
reused slot with the first record's casing and appends `CacheItem::Date`; `ItemKey::identity` keeps
date items apart from numbers; `Model::is_date_style`)

## Shared-item flags Excel accepts for mixed fields
Rule: Excel for the web refuses a cache whose `sharedItems` flags contradict its items. The flags that
open are the ones Excel itself writes:
- booleans and errors count as strings;
- `containsSemiMixedTypes="0"` only when the field has no string and no blank;
- `containsString="0"` when it has no string;
- `containsMixedTypes="1"` when strings sit beside numbers;
- numbers with blanks carry `containsBlank="1"` and no `containsSemiMixedTypes="0"`;
- a date is a type of its own: `containsDate="1"`, and `containsMixedTypes="1"` beside a number. A
  field with a date carries no number flags (`containsNumber`, `containsInteger`, `minValue`,
  `maxValue`), even beside live numbers. Whether a date beside a string and no number is mixed is
  UNVERIFIED (T-4654): T4's field also held stale numbers.

As observed and still open (T-4654):
- stale items: in T1 (dates beside stale `<n>` items) Excel wrote `containsNonDate="0"`, so the stale
  numbers did not count there; in C1 (texts beside a stale `<m/>`) it wrote `containsBlank="1"`, so
  the stale blank did. T1's `containsMixedTypes="1"` may come from the stale numbers.
- `minDate` is the least date, but `maxDate` is below it in every date field: T1, T4 and T5
  `1900-01-10T12:50:04` beside `minDate` 2023-03-15T00:00:00, T2 `1899-12-31T03:45:03`, T3
  `1899-12-31T00:03:04`.

Evidence: T-4122 runs (bool+text, error+text and number+blank refused with other flags, rendered with
these); Excel-saved caches in the LibreOffice fixtures (oss-engines.md §5).
probes/t4612-pivot-date-items-and-casing (excel-graph, 2026-11-06, T-4612), sidecar `cache_fields`
`shared_flags`:
- T5 (`<d>`, `<n>`, `<n>`, all holding records): `containsDate="1"`, `containsMixedTypes="1"`,
  `containsSemiMixedTypes="0"`, `containsString="0"`; no `containsNumber`, `minValue` or `maxValue`;
  `minDate` 2023-03-15T00:00:00, `maxDate` 1900-01-10T12:50:04.
- T4 (`<d>`, `x`, `<d>` and two stale `<n>`): `containsDate="1"`, `containsMixedTypes="1"`; the
  string flags at their defaults; no number flags.
- T1, T2, T3 (`<d>` items and stale `<n>` ones): `containsDate="1"`, `containsMixedTypes="1"`,
  `containsNonDate="0"`, `containsSemiMixedTypes="0"`, `containsString="0"`, and the `minDate` and
  `maxDate` above.
- C1 to C3 (one text item and a stale `<m/>`): `containsBlank="1"` only.
Status: builder observation (T-4122 runs) for the refused files, not committable as a probe: Excel
saves nothing for a file it refuses. The builder writes these flags (pivot_definition.py
`shared_items`), and every filled pivot probe opens with them. The date flags are VERIFIED
(excel-graph) for the fields above; a date beside a string without numbers, stale items,
`containsNonDate` and `minDate`/`maxDate` are UNVERIFIED (T-4654).
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`SharedItemsFlags`, written by the export, T-4176)

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
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`FieldReader`)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`item_cell`: how an item renders; T-4149)
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`Order`: the display order of a computed
result)

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
- roles: the outer row field, the inner row field, a column field, a page field, the data field, an
  unused source field and a calculated field;
- positions in the cache: first (R1), middle (R3, R4) and last (R6, and t4128 K's calculated field).

For the roles where `outline` was varied apart from `compact` (the outer row field, the column field
and the data field), only `compact` counts and `outline` makes no difference. With every field
`compact="0"`, an outer row field (t4609 L3), a column field (L7) and the data field (t4123 T3, T4)
left at the default `outline` still give the field's name. With `compact` at the default,
`outline="0"` on the outer row field (L4) or the column field (L8) still gives `Row Labels`.
`outline` was never varied apart from `compact` on the inner row field (Product is
`compact="0" outline="0"` in every L sheet), a page field, an unused field or a calculated field.
The second row field's header is its name (`Product`) whenever it keeps its own column.

When the field name wins, `rowHeaderCaption` is ignored: L5, with every field `compact="0"` and
`rowHeaderCaption="Regions"`, shows `Region` at A3. With a column field, `Row Labels` (or the name)
sits on the row below the data caption (L1!A4, L7!A4), and the column field's header beside it reads
`Column Labels` when the column field's `compact` is at the default (L1!C3 with both attributes at
the default, L8!C3 with `outline="0"`) and its name `Channel` when every field is `compact="0"`
(L7!C3). Whether a non-compact column field reads its name while another field is at the default is
not probed.

The outer row field's two attributes split the layout of R1 (both at the default) between them:
- `compact` and `outline` both on put the inner field in the outer field's column: R1's location is
  A3:B11 and firstDataCol 1. Either off gives the inner field its own column: L3 (`compact="0"`) and
  L4 (`outline="0"`) are A3:C11 with firstDataCol 2, like R2 to R6;
- `outline` alone decides the lines and the subtotals. With it on (R1, L3) each outer item has a
  line of its own carrying its subtotal (L3!A4 `East` with C4 22 and B4 blank, A8 `West` with C8 27)
  and there are no `<item> Total` lines; with it off (L4) the item shares its first child's line and
  `East Total` (A7) and `West Total` (A10) follow, as in R2 to R6.

Evidence (excel-graph, 2026-11-06):
- probes/t4583-pivot-row-labels A3 (T-4583). The cache fields are Region, Product, Sales, Units and
  Risky; every field carries `compact="0" outline="0"` except the one named:
  - R1 Region (outer axis, first): `Row Labels`;
  - R2 Product (inner axis): `Row Labels`;
  - R3 Units (unused, middle): `Row Labels`;
  - R4 Sales (data, middle): `Row Labels`;
  - R6 Risky (unused, last): `Row Labels`;
  - R5 (none left at the default): `Region`.
- probes/t4609-pivot-row-labels-2 (T-4609). The cache fields are Region, Product, Channel and Sales;
  every pivot is tabular with rows Region, Product and Sum of Sales, and every field carries
  `compact="0" outline="0"` except as named:
  - L1 column field Channel at the default: A4 `Row Labels`, C3 `Column Labels`;
  - L2 page field Channel at the default (location A4): A4 `Row Labels`; L6, the same page field
    non-compact: A4 `Region`;
  - L3 Region `compact="0"`, `outline` at the default: A3 `Region`, outline lines, A3:C11;
  - L4 Region `outline="0"`, `compact` at the default: A3 `Row Labels`, tabular lines, A3:C11;
  - L5 none at the default, `rowHeaderCaption="Regions"`: A3 `Region`;
  - L7 column field Channel `compact="0"`, `outline` at the default: A4 `Region`, C3 `Channel`;
  - L8 column field Channel `outline="0"`, `compact` at the default: A4 `Row Labels`,
    C3 `Column Labels`.
- probes/t4123-pivot-capture A4 (T-4123): T and T4, with Sales at the default, give `Row Labels`; T2
  and T3, with every field non-compact, give `Region`.
- probes/t4128-pivot-spike K!A4 (T-4128): a calculated field at the default gives `Row Labels`.
- probes/t4137-pivot-captions K5!A3 and K6!A4 (T-4137): unused fields at the default give `Row Labels`.

oracle-spike.md run 4, whose uncommitted hand-written XML set `compact="0"` on the Sales field, showed
`Region` too.
Correction: T-4123 first read the rule as "the data field's `compact`". T-4137 narrowed it to the
non-axis fields, and T-4583's R1 and R2 show that axis fields count too. T-4583 left R1's layout to
"`compact` and `outline` both on"; T-4609's L3 and L4 split it as above.
Status: VERIFIED (excel-graph) for the roles, positions and attributes probed. UNVERIFIED (probe
pending: T-4660): the column header of a non-compact column field while another field is at the
default, and `outline` varied apart from `compact` (either way round) on the inner row field, a page
field, an unused field or a calculated field.
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (the outer field's own line from
its `outline`, and in `label_columns` the shared column from `compact` and `outline` both on;
T-4147, checked against every L sheet by T-4609)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (the header text, every probe
above reproduced; T-4149)

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

- The headers follow the lines: each column line labels its levels from `r` on, one header row
  per column level below the location's top row (t4138 G5: `A` in C5, `Web` and `Store` in C6
  and D6, `A Total` in E5); each row line labels its levels from `r` on in its level's label
  column, compact form putting several levels in one column (G2). The sidecars hold values
  only, so a label cell's indentation is not evidenced; the engine writes the item's text alone.
- A single data field with Values on no axis puts its caption at the corner with fields on both
  axes (C!A3), over the data column with no column field (t4138 G1!C4, G8!B8), and as the label
  of the only row with no row field (G7!B6). With Values on an axis, its items are the data
  fields' captions, and its grand-total lines read `Total <data field>` (G6!B11:B12, t4135
  A_sum!N4:S4), with no `grandTotalCaption` set; whether that caption replaces the text there is
  unprobed (the engine keeps `Total <data field>`; UNVERIFIED, T-4650).
- Items render as in "Items show in the order the pivot field lists them" (t4134 I2, I4, I5).

Evidence: probes/t4123-pivot-capture, the `pivots` cells of its sidecar (excel-graph, 2026-11-06,
T-4123); t4138-pivot-geometry G1 to G9 (T-4138); t4135-pivot-aggregates (T-4135). The engine's
render reproduces every cell of the probe pivots listed in its tests (t4123 C and T to T4 but
T's show-values-as column D, t4138 G1 to G9, t4137 K1 to K6, t4583 R1 to R6, C1, C2, t4609 L1 to
L8, t4134 I1, I2, I4, I5, t4135 all twelve, t4589, t4614, t4616, and t4622 but East's custom Count subtotal
values over an error, a result rule of T-4168's, T-4636). Number formats are PV2's (T-4169).
The engine's own readings, not probed: a number or date item's subtotal label (`number_to_text`
of the value, then ` Total`); a subtotal label with Values inside its field (`<item> <data
field>`); the grand-total caption with Values on its axis (always `Total <data field>`, even with
`grandTotalCaption`); the
header over a Values level in a label column of its own (left empty). `rowHeaderCaption` beside
the all-`compact="0"` rule is settled: the field name wins (t4609 L5!A3 `Region`; Caption
attributes below).
Status: VERIFIED (excel-graph) for the probes listed. UNVERIFIED (probe pending: T-4657): number
and date item subtotal labels; (T-4650): the Values-inside and Values-axis captions and header.
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`render`; T-4149)

## Caption attributes, empty and error captions, page captions, custom subtotal labels
Rule:
- `rowHeaderCaption` replaces `Row Labels` (K1!A4 `Regions`). In a tabular pivot whose every field
  is `compact="0"` it is ignored and the field name shows (probes/t4609-pivot-row-labels-2 L5!A3
  `Region`; see "A tabular pivot's first row header..." above).
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
  several items stay visible (K4!B2:B4). What a page field shows when its hidden items leave one
  item visible, and a selected number, boolean or blank item, are unprobed: the engine shows
  `(Multiple Items)` and the item as a row label renders it (UNVERIFIED, T-4657).
- Custom subtotal functions label each subtotal line `<item> <Function>`: `East Sum`, `East Count`
  (K5!A7:A8), with Count counting records (4). `countA` is labelled `Count` too
  (probes/t4622-pivot-count-subtotals CA!A6; see "Which function the countA and count custom
  subtotals compute").
- With a single data field and no column field (tabular, K5) there is no Values row. `Row Labels` is
  at the location's corner (K5!A3), the inner row field's name beside it (B3 `Product`), and the data
  field's caption above the data column (C3 `Sum of Sales`). With a column field the corner holds the
  data caption instead (K2!A3, K3!A3; previous entry).

Evidence: probes/t4137-pivot-captions (excel-graph, 2026-11-06, T-4137). Default data-field names
(`Sum of`, `Count of`...) are written by the file, not chosen by Excel, so a file cannot probe them;
creating a pivot in the UI is where Excel picks them (T-2327 keeps that question). For other
aggregates and `#DIV/0!` under `errorCaption`, see "errorCaption replaces every error value" below.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`render`, `page_area`; T-4149)

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
independently confirmed (probe pending: T-4584). Page grids filled down with `pageWrap` are
T-4650's (G8 cannot tell the orders apart).
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`page_area`: `rowPageCount`
rows ending one row above the location, three columns per page column; T-4149)

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
The engine's layout (T-4147) reproduces the saved `location`, `firstHeaderRow`, `firstDataRow`,
`firstDataCol`, `rowPageCount` and `colPageCount` of every probe pivot in scope (listed under "Layout
lines" below). Page fields: with no `pageWrap` and the default order, every page field is in one column
(t4137 K4: 3 by 1); with `pageWrap` and `pageOverThenDown`, `pageWrap` fields per row (t4138 G8: 2 by
2). A row field shares its column with the next level only with `compact` and `outline` both on
(t4583 R1); with both off (R2), `compact="0"` alone (t4609 L3) or `outline="0"` alone (L4) the next
level has its own column.
Status: VERIFIED (excel-graph) for the table and the T-4147 reproduction above; the refusal is a builder
observation. UNVERIFIED (probe pending: T-4650): `firstHeaderRow` other than 1, and the page grid's
other two orders (G8's 2 by 2 cannot tell them apart).
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`geometry`, `location`; T-4147)
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`geometry`: the saved offsets typed
in `PivotTable::geometry`, T-4147)

## Layout lines: rowItems and colItems per form
Rule: Excel saves each axis's lines as `<i t r i><x v/></i>`. Each axis is its fields in order, with
the Values field (`x="-2"`) as one more level whose items are the data fields; a line's tuple is one
`x` per level down to its own, `r` counts the leading `x`s it shares with the line before (never its
own last one), and `<x>` lists the rest. An `x` is the item's position in the pivot field's `<items>`,
or the data field's index at the Values level (`i` repeats it).
- Every tuple of one item per level is a `t="data"` line.
- On rows, an item of a field in outline form (`outline` on, which compact form also has) has a line
  of its own above its children, which carries its subtotal values (with `subtotalTop`, the
  default). An item of a tabular field (`outline="0"`) shares its first child's line, and its
  subtotal lines follow its children. The form is per field and follows `outline` alone: an outer
  field left at the default inside a tabular pivot is outline form (t4583 R1), and so is one with
  only `compact="0"` (t4609 L3), while one with only `outline="0"` is tabular (L4).
- On columns every item shares its first child's line, and subtotal lines follow the children.
- A field has subtotal lines only when a field other than Values is inside it. They are one
  `t="default"` line, or one line per custom subtotal in the field's order (`t="sum"`, `t="count"`).
- The grand total is one `t="grand"` line `<x/>` per data field when Values is on the axis, one
  otherwise, and none on an axis whose only level is Values.
- An axis with no level at all saves one empty line `<i/>`.

Evidence (excel-graph, 2026-11-06), the sidecars' `row_items` and `col_items`, all equal to the engine's
lines (T-4147):
- probes/t4123-pivot-capture C, T to T4 (compact with a column field; tabular with Values on columns);
- probes/t4138-pivot-geometry G1 to G9 (G2 compact, G3 outline, G4 tabular with three row fields, G5
  two column fields with `default` lines after their children, G6 Values on rows with two `grand`
  lines, G7 and G9 `<i/>`, G8 page fields only);
- probes/t4137-pivot-captions K1 to K6 (K5 tabular `sum` and `count` lines), t4583-pivot-row-labels
  R1 to R6, C1, C2, t4609-pivot-row-labels-2 L1 to L8, t4134-pivot-items I1, I2, I4, I5,
  t4135-pivot-aggregates (all twelve), t4589-pivot-total-order, t4614-pivot-total-order-2 and
  t4616-pivot-total-order-3 (every sheet);
- the lines of probes/t4128-pivot-spike S (its #SPILL! location is the grid's, T-4151);
- the Excel-saved pivot of crates/rustsheet-xlsx/tests/example.xlsx: three compact row fields whose
  `<items>` list the cache items out of order, so `x` is the position in `<items>`, not the cache index.

Not compared, because the engine-side harness cannot rebuild their cache or tree yet: t4128 R, t4134
I3, t4582 J1, J2 and t4585 N1 have formulas in their sources, which the case file does not evaluate
(T-4152 runs them through the engine); t4582 J3 and J4 depend on date items against cached numbers
(T-4612); t4128 G is grouped (T-4171) and t4128 K has a calculated field, a Top-N filter and a
descending sort (T-4174, T-4165).

The engine's own readings, not probed:
- `subtotalTop="0"` on an outline row field: the item's own line holds no values and a `default` line
  follows its children; custom subtotals in outline form follow the children too; with
  `defaultSubtotal="0"` and no custom subtotal there is no subtotal line, and `defaultSubtotal="1"` with
  custom subtotals is the default line alone; `rowGrandTotals="0"` drops the grand line of `rowItems`
  and `colGrandTotals="0"` that of `colItems` (an Excel-saved corpus file shows the latter,
  oss-engines.md §0.5, not committed);
- the Values field outer or in the middle of an axis: a field with Values inside gets one subtotal line
  per data field after its children; on rows with `outlineData`, each data field has a line of its own
  above the inner items;
- an item the pivot field does not list (Excel's files list every one) takes the positions after the
  listed items, in display order.

Status: VERIFIED (excel-graph) for the probes listed. UNVERIFIED (probe pending: T-4649): subtotal
placement and grand totals off; (T-4650): the Values field away from the innermost level and
`outlineData`; the unlisted-item positions are an engine convention for files Excel does not write.
Blank rows, items with no data and collapsed items are T-4168's.
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`lay_out`, `AxisPlan::lines`; T-4147)

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
UNVERIFIED (probe pending: T-4633): a cell whose records hold no
number at all (only text, blanks or booleans) is unprobed; the engine gives it the worksheet
functions' answers (Sum, Max, Min and Product 0, Average, StdDev and Var `#DIV/0!`, StdDevp and Varp
`#DIV/0!` with no number). A date in a value field counts as its serial number, also unprobed. When
a cell or total covers two different errors, the engine shows the first in accumulation order, also
unprobed (T-4633).
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`compute`, `PivotResult::value_as`), crates/rustsheet-engine/src/pivot/accumulate.rs (`Accumulator::finish`)

## Summation order of Sum, Average and Product totals
Rule:
- **Item cells.** A cell accumulates its records in record order.
- **Sum, when Sum is the only aggregate (2-D).** A row total adds the row's cells in the column
  field's display (first-seen) item order. A column total adds the column's cells in the row field's
  display order. The grand-grand total adds the row totals (the Grand Total column) down the rows, in
  display order. These are sums of the child cells, not of the records.
- **Sum beside Average (2-D).** The row and column totals accumulate the records item by item, in the
  display item order of the field being totalled and in record order within each item. Child cells are
  not used. So the same data can give a different Sum total once an Average field is added.
- **Average (alone or beside Sum, 2-D).** A row or column total divides the same item-by-item
  accumulation by the count.
- **Var and StdDev, alone or together.** Every Var cell and total is the textbook formula
  (Σx² − (Σx)²/n) / (n − 1). Its two sums accumulate the records item by item in display item order,
  in record order within each item. StdDev is the square root of that Var.
- **Product (alone, one axis field).** The total multiplies the records item by item in display
  item order, in record order within each item.
- **Totals over one axis field in mixed pivots.** In a pivot that mixes Sum with other aggregates,
  a total over a pivot's only axis field accumulates the records item by item in display item order
  (T-4614 S2, T-4589 S1).

What is open (T-4634):
- which mixes switch Sum from the cells to the records;
- whether a Sum-only total over one axis field sums the item cells or accumulates the records (no
  probe separates them);
- the grand-grand total of a mixed 2-D pivot (one data point);
- which of several fitting readings gives the 2-D Var grand-grand total.

Evidence (excel-graph, 2026-11-06):
- probes/t4616-pivot-total-order-3 (T-4616). D1 has 27 records, three per cell: R items first seen
  Q, X, B, and K items first seen M, Z, A. Python doubles were used to choose values for which the
  candidate totals differ. The candidates are:
  - the row's cells added in display, reverse or ascending item order;
  - records accumulated item by item in display, reverse or ascending item order;
  - record order, and reverse record order.

  | Total | cells, display | cells, reverse | cells, ascending | items, display | items, reverse | items, ascending | records | records, reverse |
  |---|---|---|---|---|---|---|---|---|
  | row Q | 19313.14 | 19313.139999999996 | 19313.14 | 19313.14 | 19313.140000000007 | 19313.14 | 19313.140000000003 | 19313.140000000003 |
  | row X | 5223.799999999999 | 5223.8 | 5223.8 | 5223.799999999999 | 5223.8 | 5223.8 | 5223.8 | 5223.8 |
  | row B | 871.3100000000001 | 871.3100000000001 | 871.3100000000001 | 871.3099999999998 | 871.3100000000001 | 871.31 | 871.31 | 871.3100000000002 |
  | column M | 4406.299999999999 | 4406.299999999999 | 4406.3 | 4406.299999999999 | 4406.299999999998 | 4406.3 | 4406.299999999999 | 4406.299999999999 |
  | column Z | 12569.289999999999 | 12569.289999999999 | 12569.289999999999 | 12569.289999999997 | 12569.29 | 12569.29 | 12569.289999999997 | 12569.29 |
  | column A | 8432.659999999998 | 8432.66 | 8432.66 | 8432.659999999998 | 8432.66 | 8432.66 | 8432.66 | 8432.66 |

  - S1 (Sum alone) shows the cells-in-display-order column: E5:E7 are 19313.14, 5223.799999999999 and
    871.3100000000001, and B8:D8 are 4406.299999999999, 12569.289999999999 and 8432.659999999998. No
    other candidate fits all six.
  - S2 (Sum beside Average, Values inside K) shows the items-in-display-order column: H6:H8 are
    19313.14, 5223.799999999999 and 871.3099999999998, and B9, D9 and F9 are 4406.299999999999,
    12569.289999999997 and 8432.659999999998. No other candidate fits all six.
  - S2's Average totals (I6:I8, C9, E9, G9) and S3's (Average alone, E5:E7, B8:D8) equal the
    items-in-display-order sums divided by 9; for example, S3 E7 is 96.8122222222222 = 871.3099999999998 / 9.
    No other candidate fits.
  - S4 (S1 transposed, K on rows) gives the same cells-in-display-order totals (E5:E7, B8:D8).
  - S5 (Sum alone over D2, 18 records) and S6 (Sum alone over D4, 36 records, six R items): cells in
    display order is the only candidate that fits every row total (S5 E5:E7, S6 E5:E10) and every
    column total in S6 (B11:D11). In S5's column totals, two other candidates also fit.
  - Grand-grand total, Sum alone. S1 E8 is 25408.25, the sum of E5:E7 in display order. On its own,
    S1 E8 fits many readings, among them the row totals added in any order and several cell and record
    orders. It refutes the sum of the column totals (25408.249999999993), the cells added row by row
    (25408.249999999996), record order (25408.250000000015) and row-major record accumulation
    (25408.25000000001).
  - S4 E8 is 25408.249999999993, the sum of S4's own row totals E5:E7 in display order. Of the
    readings tried that fit S1 E8, only that one gives 25408.249999999993 in S4. The others give
    25408.25, 25408.25000000001, 25408.250000000007 or 25408.249999999996. S5 E8 (14276.83) and
    S6 E11 (32198.6) also equal the sum of their row totals in display order. Of the candidates, only
    "row totals in display order" fits all four pivots.
  - Grand-grand total, mixed. S2 H9 is 25408.25. Among the readings that fit it are the sum of
    H6:H8 (in any order), reverse-row-item record accumulation, and several cell and record orders.
    It is not record order (25408.250000000015) or row-major accumulation in display
    order (25408.25000000001). Average's I9 is 941.0462962962963 = 25408.25 / 27 (S3 E8 too).
  - Cells: X/Z of D1 (three records) is 123.67999999999999 in S1 C6, S2 D7 and S4 B6, the record-order
    sum. Reverse order would give 123.68.
  - Var (D4, V1): V1's row totals E5:E10 and column totals B11:D11 fit only the textbook formula with
    item-by-item display-order sums. The two-pass, n·Σx² variant and Welford's update, each in display,
    reverse or ascending item order or in record or reverse record order, are refuted. Every V1 cell
    (B5:D10, two records each) fits the textbook formula.
  - Var's grand-grand total (V1 E11, 4337674.527019682) fits textbook Var over row-major (R, then K)
    display-order accumulation. About 38 other readings also fit: textbook, n·Σx², two-pass and
    Welford's update over many orders, and textbook over per-row or per-column partial sums.
    Column-major display order gives
    4337674.5270196805.
  - Var over one axis field: V3 (rows K) B8 is 4337674.52701968 and V4 (rows R) B11 is
    4337674.527019681. Each equals textbook Var accumulated item by item in its own field's display
    order. For V3 that is the only fitting candidate. The item cells V3 B5:B7 and V4 B5:B10 are
    textbook Var in record order. So the same 36 records give three different Var grand totals in V1,
    V3 and V4.
  - StdDev (V2, and V3/V4 column C): each cell and total is the square root of the textbook Var. In V3
    (C8, 2082.7084594392177), display-order textbook is the only fitting candidate. V2's grand-grand
    total E11 is 2082.7084594392186 = sqrt(V1 E11).
  - Product: P1 (D5) Grand Total B7 is 1801036823.2874622 and P2 (D6) B7 is 130884059.83325395. Each
    is the item-by-item display-order product. These were the candidates:
    - ascending value order, 1801036823.2874618 and 130884059.83325396;
    - record order, 1801036823.2874625 and 130884059.83325398;
    - the product of the item cells, 1801036823.2874613 and 130884059.83325401;
    - reverse and ascending item order, and descending value order.

    Each of those differs from the actual value in P1 or P2. This settles T-4614's P2 B6, where
    ascending value order also fitted.
- probes/t4614-pivot-total-order-2 (T-4614). D1 holds the records Z 6.5, A 0.1, Z 2.3, A 8.9, Z 2.4,
  A 9.4, with Z first seen.
  - S2 (rows K, Sum beside Average): Grand Total Sum 29.6 (B7), and Average 4.933333333333334 = 29.6 / 6
    (C7). The candidates were record order 29.599999999999994, reverse 29.600000000000005, ascending
    item order 29.599999999999998, and display item order 29.6.
  - S3 (Sum alone, K on columns): Grand Total 29.6 (D5). The sum of the item cells,
    11.200000000000001 + 18.4, is also 29.6, so S3 does not separate cells from records.
  - P2 (D2, Product alone): Grand Total 15743.706264 (B6), display item order. Record order gives
    15743.706264000002.
- probes/t4589-pivot-total-order S1 (T-4589): seven aggregates over records A 0.2, B 0.7, A 3.3, B 1.1.
  - The Sum Grand Total is 5.300000000000001 (B7), display item order. The item cells would give 5.3,
    and record order 5.299999999999999.
  - The Var Grand Total is 1.8691666666666658 (F7), textbook Var in display item order. Record order
    gives 1.8691666666666673, and two-pass 1.8691666666666664. StdDev (E7) is 1.3671747023210552.
- probes/t4135-pivot-aggregates (T-4135). On the A_sum sheet every data field is a Sum. East's records
  in Dec are 0.3 (B), 0.2 (A) and 0.1 (A), with Kind on columns shown B, then A.
  - A_sum!S6 is 0.6000000000000001, the sum of East's cells (0.3 + 0.30000000000000004). Display item
    accumulation would give 0.6. This fits the Sum-alone rule.
  - A_average!S6 is 0.19999999999999998 = 0.6 / 3, the item-by-item accumulation, which fits the
    Average rule.

Correction: T-4135 first recorded Sum as the sum of the children. T-4614 then recorded item-by-item
accumulation for totals over one axis field. T-4616 shows that both happen, depending on the pivot's
data fields: Sum alone in 2-D uses the cells, and Sum beside Average uses the records.

Status: VERIFIED (excel-graph) for these totals:
- 2-D Sum totals over one field, Sum alone and beside Average;
- the Sum-alone grand-grand total;
- Average, Var and StdDev row and column totals;
- Var and StdDev over one axis field;
- Product over one axis field;
- totals over one axis field in mixed pivots.

UNVERIFIED (probe pending: T-4634): which data-field mixes switch Sum to the records, a Sum-only total
over one axis field, and the mixed and Var grand-grand totals.
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`compute`, `PivotResult::value_as`), crates/rustsheet-engine/src/pivot/accumulate.rs (`Accumulator::finish`). The engine reads the open points this way, each UNVERIFIED
(T-4634): Sum uses the child cells only when every data field is a Sum; a Sum-only total over one axis
field sums the item cells; the mixed 2-D grand-grand Sum and Average sum the row totals' record sums
(t4616 S2 H9 and I9, S3 E8 fit it); the Var and StdDev grand-grand total is row-major (t4616 V1 E11
fits it); and a Sum-only cell that is a total on both axes (a subtotal by a subtotal or by a grand
total, with several fields on an axis) sums its row children first, which is probed only for the
grand-grand total. Varp and StdDevp use (n·Σx² − (Σx)²)/n² (bit-identical when divided by n twice).
Of the candidates tried (textbook /n, Σx²/n − mean², two-pass, Welford, (Σx² − Σx·mean)/n) it is the
only family that fits probes/t4135-pivot-aggregates A_stdDevp!N6 and M8 (t4589 S1 G5:H7 fits it too);
the order of their totals is assumed to be Var's.

## errorCaption replaces every error value
Rule: with `showError` and `errorCaption`, every error value a pivot cell would show becomes the
caption, as text. That includes `#DIV/0!` from StdDev and Var over one value, not only errors from the
data.
Evidence: probes/t4135-pivot-aggregates EC (excel-graph, 2026-11-06, T-4135):
- East (B5:L5) shows `ERR` for every aggregate except Count (3) and CountNums (2).
- West (B6:L6) shows `ERR` for StdDev and Var, which have one value, and 0 for StdDevp and Varp.
- The Grand Total row (B7:L7) is `ERR` everywhere; see the next entry for Count and CountNums.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`Cells::value`; T-4149)

## A Count total over an error field is the error when the pivot mixes aggregates
Rule: Count and CountNums never show an error in an item's cell. When the pivot's data fields mix
aggregates (Count or CountNums beside Sum or Average), though, every total of a Count or CountNums
that covers a record holding an error is that error: `#N/A`, or the errorCaption when one is set.
This covers subtotals of an outer row field over an inner one, row and column grand totals, and the
grand total. Totals that cover no such record stay counts. With one data field, with only Count fields,
or with Count beside CountNums, every total is a count. A column field makes no difference.
Evidence (excel-graph, 2026-11-06):
- probes/t4616-pivot-total-order-3 (T-4616), over D3: East A (Err 10), East B (Err `#N/A`), East A
  (3), West A (7), West B (5); rows Region, then Kind; no showError:
  - I1, Count of Err beside Sum of Num: East's subtotal is `#N/A` (B5), while its items A and B are 2
    and 1 (B6, B7). West's subtotal is 2 (B8), and the Grand Total is `#N/A` (B11).
  - I2, CountNums of Err beside Sum of Num: East's subtotal is `#N/A` (B5), with B at 0 (B7). West
    is 2 (B8), and the Grand Total is `#N/A` (B11).
  - I3, Count of Err alone: East 3 (B4), Grand Total 5 (B10). I4, CountNums of Err alone: East 2 (B4),
    Grand Total 4 (B10).
  - M1, Count of Err beside CountNums of Err, rows Region: East 3 and 2 (B5, C5), Grand Totals 5 and
    4 (B7, C7). M2, Count of Err beside CountNums of Num: Grand Totals 5 and 5 (B7, C7). M3, M1's
    fields with rows Region and Kind: East's subtotal 3 and 2 (B5, C5), Grand Totals 5 and 4 (B11, C11).
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
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`compute`, `PivotResult::value_as`), crates/rustsheet-engine/src/pivot/accumulate.rs (`Accumulator::finish`). The engine treats any aggregate other than Count and CountNums as
mixing; Count beside only Max, Min, Product or the variances is unprobed (UNVERIFIED, T-4634).

## Record filters: page selections and hidden items
Rule: a page field's selected item keeps only the records holding it, and a hidden item (`item h="1"`)
of a page field drops its records, before anything is accumulated; the pivot shows only what is kept.
The engine applies hidden items of row and column fields the same way (unprobed, see Status).
Evidence: probes/t4137-pivot-captions K4 (excel-graph, 2026-11-06, T-4137): Product selects `A` and the
page field Units hides 1 and 2, so of A's records (Sales 10, 20, 4 with Units 1, 2, 6) only 4 is left,
and the pivot shows 4 (K4!A7).
Two readings are the engine's own, not probed:
- a `pageField@item` that names no data item of its field (an index past the field's `<items>`, one
  naming a subtotal or blank line, or a malformed number, which T-4139's import reads as absent) is
  `(All)`;
- a hidden item on a field that is on no axis filters nothing.

Status: VERIFIED (excel-graph) for a page field's selection and hidden items (K4) only. UNVERIFIED
(probe pending: T-4164): hidden items of a row or column field, and the totals over them (SPEC-0032
stage 5 leaves filters and totals to T-4164). UNVERIFIED (probe pending: T-4633): the two readings
above.
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`filter`)

## A pivot result is bounded in work and nodes
Rule: computing a pivot's result (its filtered records, axis trees and accumulators) spends at most
268,435,456 work units (`MAX_PIVOT_WORK`) when run on its own, or what the caller's budget has left: one
unit per record and stage, per item ordered, per sort step, per node or accumulator made and per value
fed to an accumulator. It holds at most 10,000,000 nodes (`MAX_PIVOT_NODES`): axis nodes plus one
accumulator per (row node, column node, data field) some record reaches. Past either the pivot has no
result (`PivotComputeError::Limit`), where Excel would render it.
Its cells (`render`, T-4149) hold at most 134,217,728 bytes of text together
(`MAX_PIVOT_TEXT_BYTES`, about 32 per cell of the cell cap), since each cell copies its label;
past it the render fails with `Limit`, where Excel would show the pivot.
Its layout (`lay_out`, T-4147) draws on the same work units, one per line and `x` made, and has no
result past 4,000,000 cells (`MAX_PIVOT_CELLS`: the location's rows times columns plus two per page
field, checked from the line counts before any cell is made) or when the location would pass the
sheet's last row or column. What Excel does with a pivot that does not fit the sheet is not probed.
The values stay without a layout, so GETPIVOTDATA still has them.
Evidence: none for Excel; this is a resource limit (SPEC-0032 "Untrusted-input limits", T-4142).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`MAX_PIVOT_WORK`, `MAX_PIVOT_NODES`, `charge`,
`count_nodes`)
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`MAX_PIVOT_CELLS`, `location`; T-4147)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`MAX_PIVOT_TEXT_BYTES`; T-4149)

## Pivots without refreshOnLoad, and source edits
Rule: unknown. Two things are unprobed: whether Excel for the web re-renders a pivot whose cache lacks
`refreshOnLoad` when it opens the file (saved cells or `rowItems` stale or absent), and whether it
re-renders a pivot after its source is edited with no refresh.
Evidence: none. Every rendered probe so far is `refreshOnLoad` (case builder v2).
Status: UNVERIFIED (probe pending: T-4586)

## Pivot field names for duplicate and blank source headers
Rule: on refresh Excel names each cache field after its source header, made unique ignoring case:
- Text headers are named first, left to right. A header that no earlier name equals keeps its text,
  whitespace included (`" "`). Otherwise it takes the first `<header><k>`, k = 2, 3, ..., that no
  name taken so far equals, in the header's own casing. The suffixed names count as taken, and so
  does a real header that already ends in a number: `Sales`, `Sales2`, `="Sales"` save as `Sales`,
  `Sales2`, `Sales3`, but `Sales`, `="Sales"`, `Sales2` save as `Sales`, `Sales2`, `Sales22`.
- A blank header, an empty cell or empty text (`=""`), is `Field<n>`, n its 1-based position in the
  source, not its sheet column. It is named after all text headers, later ones too, and made unique
  the same way: `X`, `=""`, `Field4` save as `X`, `Field42`, `Field4`.
- A constant header typed twice is renamed like a formula one (`Sales`, `Sales`, `SALES` save as
  `Sales`, `Sales2`, `SALES3`).
- A whole-text header `ß` blocks a later `SS`: `é`, `="É"`, `ß`, `="SS"`, `="s"&"s"` save as `é`,
  `É2`, `ß`, `SS2`, `ss3`. Items do not equate them: in the same source the Region items `ß` and
  `SS` stay apart, as do `Straße` and `STRASSE`, while `é` and `É` are one item (casing of the
  first, `é`). Whether `ß` inside a longer name blocks its `ss` spelling, and whether names compare
  by the `=` collation, are open (below).
- A record cell holding `=""` is a text item of its own, apart from an empty cell's `(blank)` item;
  its row label is an empty cell. It is listed after `(blank)`: a, (blank), then the `""` item. That
  fits new items being appended on refresh ("Items show in the order the pivot field lists them;
  new items are appended"), since the builder caches formula cells as blank, so `""` is the one new
  item.

Evidence: probes/t4585-pivot-field-names (excel-graph, 2026-11-06, T-4585): headers D!A1:F1 Region,
Sales, `="Sales"`, `=""`, `="region"`, `="Sales"&""` save as Region, Sales, Sales2, Field4, region2,
Sales3. probes/t4627-pivot-field-name-collisions (excel-graph, 2026-11-06, T-4627), sidecar `pivots`
cache_fields of the pivot on sheet P_<source>:
- SufA!A1:D1 Region, Sales, Sales2, `="Sales"` → Region, Sales, Sales2, Sales3;
- SufB!A1:D1 Region, Sales, `="Sales"`, Sales2 → Region, Sales, Sales2, Sales22;
- FldA!A1:D1 Region, Sales, Field4, `=""` → Region, Sales, Field4, Field42;
- FldB!A1:E1 Region, Sales, X, `=""`, Field4 → Region, Sales, X, Field42, Field4;
- Off!B1:G1 Region, Sales, X, `=""` (sheet column E), Y, Z → Region, Sales, X, Field4, Y, Z;
- Empty!A1:E1 Region, Sales, X, an empty cell, Y → Region, Sales, X, Field4, Y;
- Const!A1:D1 Region, Sales, Sales, SALES (constants) → Region, Sales, Sales2, SALES3;
- Space!A1:E1 Region, Sales, `=" "`, `=" "&""`, X → Region, Sales, `" "`, `" 2"`, X;
- Fold!A1:G1 Region, Sales, é, `="É"`, ß, `="SS"`, `="s"&"s"` → Region, Sales, é, É2, ß, SS2, ss3.
  Its pivot (rows Region over é, É, ß, SS, ss, Straße, STRASSE with Sales 1, 2, ..., 64) shows
  P_Fold!A4:B8 é 3, ß 4, Straße 32, SS 24, STRASSE 64;
- Rec!A2:A6 a, `=""`, empty, `=""`, a with Sales 1, 2, 4, 8, 16: the rendered cells P_Rec!A4:B6 are
  a 17, (blank) 4 and an empty label cell 10 (the sidecar holds no shared items, so the rendered
  cells are the evidence).

Empty and Const are built with the case builder's `cache_names` (pivot_definition.py), which saves
placeholder names that Excel replaces on refresh, as it replaces formula headers' text.

The probed sources cannot tell whether the suffix search restarts at 2 or starts at the count of
earlier equal headers; both give every name above.

Open (T-4640):
- number, boolean and error headers;
- `ß` inside a longer name (`Straße` against `STRASSE`): the engine's `name_key` folds it to `ss`
  anywhere, an unprobed extrapolation of the one whole-text `ß` header;
- whether names compare by Excel's `=` collation, where `ß`/`ss` is one of several expansions
  (coercion.md "Collation equality in lookups and criteria"): `æ`/`ae`, `œ`/`oe`, `ĳ`/`ij`, `þ`/`th`,
  `á`/`a`+U+0301, U+00AD ignored;
- other case folds: `ẞ` (the engine folds it to `ß` and so treats `ẞ`, `ß`, `SS` and `ss` as one
  name, though exact MATCH keeps `ẞ = ß` apart from `ss`: lookup.md "MATCH, VLOOKUP and HLOOKUP
  exact-match keys fold case one character to one"), `ǆ`/`Ǆ`, `ı`/`I`, `ς`/`σ`, ligatures;
- a blank's `Field<n>` meeting a suffixed name from a later duplicate, and `Field4` against a typed
  `field4`;
- carried over to T-4640 from code pointers: whether a refresh matches new fields to the old
  cache's by name, case or position (`refresh_builder`, pivot/refresh.rs), and whether
  GETPIVOTDATA's item `"#N/A"` names an error item (functions/lookup_and_reference/pivot_index.rs).

Status: VERIFIED (excel-graph) for the rules above; the open points are UNVERIFIED (probe pending:
T-4640)
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141, T-4627) (`field_names`,
`name_key`)
