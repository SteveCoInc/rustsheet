# Web functions (ENCODEURL, FILTERXML, WEBSERVICE)

WEBSERVICE is out of scope (decision 0027; eval/catalog/functions.yaml).

## FILTERXML is `#VALUE!` in Excel for the web
Rule: in Excel for the web, `FILTERXML("<a>1</a>","//a")` (well-formed XML, valid XPath) is
`#VALUE!`. Microsoft's FILTERXML page says the function relies on Windows and does not return
results on Mac; what desktop Excel on Windows returns is not yet measured.
Evidence: probes/catalog/function-name-forms A7 (ERROR.TYPE 3 = #VALUE!); probes/catalog/availability-14
A7 (`FILTERXML($Z$1,$Z$1)` is recognised, not #NAME?) (oracle: excel-graph, 2026-10-07, T-0040).
Quirk: an error in the first argument propagates instead of the `#VALUE!` (gen-m6/abcfd9979f316267 A9
`FILTERXML(D4&NA(),..)` is `#N/A`; gen-m7/4629e931955aa4b8 A34 gives DROP's `#CALC!`).
An error only in the second argument propagates too
(`FILTERXML("x",NA())` is `#N/A`), and with errors in both the first argument wins, left to right
(`FILTERXML(1/0,NA())` is `#DIV/0!`, `FILTERXML(NA(),1/0)` is `#N/A`).
Evidence: probes/filterxml-error-arguments A1:A6 (oracle: excel-graph, 2026-10-15, T-1570).
An array argument lifts: `FILTERXML({"<a>1</a>";"<a>2</a>"},"//a")` spills two `#VALUE!` (probes/filterxml-evidence-floor D27:D28, T-1704). Probed too (probes/filterxml-array-forms, excel-graph, T-1722): a range as the first argument (`FILTERXML(Z1:Z2,"//a")`) and an array as the second (`FILTERXML("<a/>",{"//a","//b"})`) give `#VALUE!` per element; an error element passes through while the others stay `#VALUE!` (`FILTERXML({"x";#N/A},"//a")` is `#VALUE!`, `#N/A`); a scalar error next to an array broadcasts (`FILTERXML(NA(),{"//a";"//b"})` spills two `#N/A`, `FILTERXML({"<a/>";"<b/>"},NA())` too); a lambda first argument is `#VALUE!` even with `NA()` second (`FILTERXML(LAMBDA(x,x),NA())`).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)
Implemented in: crates/rustsheet-engine/src/functions/text/filterxml.rs (T-1255, decision 0039); no XML is parsed.

## ENCODEURL is `#VALUE!` in Excel for the web
Rule: in Excel for the web, ENCODEURL is `#VALUE!` for every argument probed: `"a b&c"`, `"abc"`, a text cell, `1`
and `TRUE`. It lifts over an array or a range, and each element is `#VALUE!` (`ENCODEURL({"a b","c"})` spills two). As with
FILTERXML, the function seems not to work on the web. The engine follows the golden (decision 0039 rule 4) and encodes nothing; Microsoft's page
describes percent-encoding (`"a%20b%26c"`), which desktop Excel may do. An error argument propagates as it came (unprobed).
Evidence: probes/info-logic-argument-modes-2 A1:A6, B5; probes/info-logic-argument-modes A15:B15 (oracle: excel-graph, 2026-10-09, T-1088).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). Implemented in: crates/rustsheet-engine/src/functions/text/encodeurl.rs (T-1219).
Tasks: T-1088, T-1219

## Evidence floor probes (T-1628)
Rule: on excel-graph every ENCODEURL cell probed is `#VALUE!` whatever the text (reserved characters, non-ASCII,
a surrogate pair, `""`, numbers, booleans, a date, a cell reference, a range and an array literal, which spill as
`#VALUE!`), including `ENCODEURL(ENCODEURL("a b"))` and `LEN(ENCODEURL("a b"))`; an error argument propagates
(`ERROR.TYPE(ENCODEURL(1/0))` is 2). Every FILTERXML cell probed is `#VALUE!` too: well-formed and malformed XML,
no-match XPath, several matches, attributes, `count()`/`string()`, numeric text, a non-text argument and an array argument.
Evidence: probes/encodeurl-evidence-floor, probes/filterxml-evidence-floor (oracle: excel-graph, 2026-10-16, T-1628).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). The web results say nothing about desktop
percent-encoding or XPath behavior.

## GETPIVOTDATA without a pivot table (T-1628)
Rule: with no pivot table in the workbook, `GETPIVOTDATA("Sales",ref,...)` is `#REF!` for every reference probed
(a cell outside any pivot, a range, `OFFSET`/`INDIRECT` results, extra field/item pairs, an odd number of
pairs, `""` as data_field, a numeric pivot_table or numeric item). An error in either of the first two arguments
propagates as it came (`pivot_table` `NA()` is `#N/A`, `1/0` is `#DIV/0!`; `data_field` `NA()` is `#N/A`).
`ERROR.TYPE(GETPIVOTDATA("Sales",A1))` is 4. Which error wins when both are errors, a boolean data_field, a
reference to an existing second sheet, an error inside a field/item pair (`A1,NA(),1`) and an array
data_field or pivot_table are unprobed (ours: pairs are not evaluated, an array argument gives one `#REF!`, T-1703).
Evidence: probes/getpivotdata-evidence-floor (oracle: excel-graph, 2026-10-16, T-1628).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081).
A real pivot: see the next section (T-1705).

## Pivot caches past the import limits are dropped
Rule: Excel loads a pivot of any size its grid allows. The importer drops, never truncates, a pivot cache with more
than 1,048,576 records, 16,384 fields, 1,000,000 shared items over its fields or 10,000,000 record values (records x
fields), every cache once the workbook's caches together pass 10,000,000 record values (in workbook order), and a pivot
with more than 256 fields on an axis (rows, columns or page) or 256 data fields, past the 64th pivot of a sheet, or past the 1,000th pivot
of the workbook (in sheet order; T-2394); the pivots on a dropped cache go with it. GETPIVOTDATA against a dropped pivot is `#REF!` rather than a wrong total.
A pivot is also dropped when its field lists more `<item>`s than its cache field's items plus 14 (one line of each
other `ST_ItemType`), or when the parts of its definition the model keeps verbatim (SPEC-0032 "Model", T-4139) would
pass what is left of the workbook's verbatim budget: 32 MiB (`MAX_OPAQUE_BYTES`), or the package's inflate limit
(`ImportLimits::max_uncompressed_bytes`) when that is lower, each kept entry charged its text plus its own size. A
pivot is also skipped for an unknown `axis`, `sortType`, `showDataAs`, data field `subtotal` or `item@t`, for a
`<filters>` entry it cannot type (T-4165: a `filter@fld` past the cache's fields, a `customFilter@operator` or a
`dateGroupItem@dateTimeGrouping` it does not know; before T-4165 these pivots loaded with their filters kept verbatim
and unapplied; UNVERIFIED, T-4799, what Excel does with one), and past the filter caps (more than 2,048 filters, more
than two `customFilter`s in one filter, more than 1,000,000 values in one values list: DEVIATION, docs/excel-semantics/
pivots.md "Pivot filters past the import limits"); a malformed
boolean, and a malformed number the model only keeps (`dataPosition`, `pageWrap`, `numFmtId`, `baseField`,
`baseItem`, a page field's `item` or `hier`), reads as the schema's default instead.
The limits are totals counted while parsing, before anything past them is allocated (CLAUDE.md directive 5).
The xlsx export holds each pivot cache to the same totals (records, record values per cache and across the
workbook's caches, shared items) and drops a cache past them, with its pivots and a log line, as it does a cache
with a non-finite number or an out-of-range record index (T-4176).
Part names that differ only by case or by `\` against `/` are one part, so a pivot or cache reached twice that way is skipped like any other. A
sheet that loses pivots to a cap logs one line with the count, not one per pivot.
Evidence: none for Excel; this is a resource limit (SPEC-0015 "Untrusted-input limits").
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/pivot.rs (`CacheBuilder`, `MAX_*`), crates/rustsheet-xlsx/src/import/pivots.rs, crates/rustsheet-xlsx/src/import/pivots/table.rs (`MAX_EXTRA_ITEMS`, `OpaqueEntry::cost`; `MAX_OPAQUE_BYTES` in pivots.rs), crates/rustsheet-xlsx/src/export/pivots/cache.rs (`check`)

## GETPIVOTDATA against a pivot table (T-1705)
Rule: against a compact-form pivot (one row field, an optional column field, one data field), GETPIVOTDATA
returns the value of the pivot cell the field/item pairs pick, including subtotal and grand-total cells:
- `data_field` is the source field name (`"Sales"`) or the data field's caption (`"Sum of Sales"`), matched
  without regard to case (`"sales"`). Padding is not trimmed: `" Sales "` is `#REF!`. A caption of another
  aggregate (`"Sum of Sales"` on an Average pivot) and an unknown field (`"Profit"`) are `#REF!`. A blank cell
  as `data_field` gives the grand total (one data field; `""` with no pivot is `#REF!`, see above).
- No pairs give the grand total, one pair a row or column total, two pairs their intersection in either
  order. Field and item names ignore case (`"region","east"`; `"Product","a","Region","EAST"`). An item may
  be a reference to a label cell (`A5`).
- `#REF!`: an item not in the field (`"North"`, `" East"`), a field not in the pivot (`"Market"`, or a
  source field the pivot does not show), the same field twice with different items, and a field with no
  item (an odd argument count).
- An intersection that has no source rows (shown blank) is **0**, not `#REF!` (West/C).
  The same holds for every aggregate the case builder writes: Average, Count, Max and Min pivots answer 0 on an
  empty intersection (West/C), while the non-empty intersection, row total, column total and grand total are the
  aggregate (Average East/A 7, Count 2, Max 10, Min 4) and the pair order does not matter. Evidence:
  probes/getpivotdata-empty-intersection (excel-graph, 2026-10-26, T-2404), plus gen-m6 8641fa604c5b8e14 Calc!A19,
  A25 and A39 (Count of Units; A19 names the source field `"Units"` as data_field on that Count pivot and Excel
  accepts it). CountNums, Product, Var, VarP, StdDev and StdDevP over an empty intersection are 0 too: "GETPIVOTDATA on other pivot shapes (T-2085)" below, the "Aggregates over one record or none" paragraph.
- `pivot_table` may be any cell of the pivot (`$C$6`, inside the data area) or a range covering it
  (`$A$3:$E$7`); a cell outside every pivot (`$H$30`, or the source data `Data!$A$1`) is `#REF!`.
- An array of items lifts: `"Region",{"East";"West"}` spills 22, 27 and `"Product",{"A","B","C"}` spills
  34, 12, 3.
Evidence: probes/getpivotdata-real-pivot (oracle: excel-graph, 2026-10-16, T-1705). The pivots are written
by the case builder (`pivot_tables`, tools/src/rustsheet_tools/pivot.py); excel-graph rendered them cell for
cell as the builder does (the builder also writes the rendered cells into the sheet). probes/pivot-rendered-cells
(T-2084) reads every cell of both pivots through `=P!A3` and `ISBLANK`; excel-graph opens the workbook and agrees with
the layout (caption, `Row Labels`/`Column Labels`, items, aggregates, grand totals, West/C blank: `ISBLANK` true,
reference 0). Excel rebuilds a `refreshOnLoad` pivot's cache from its source on open and renders from it: a pivot
whose cache record is stale shows the source's value (probes/t4128-pivot-spike R!B5 1004, T-4128;
docs/excel-semantics/pivots.md). That pivot's area held no cells, though. UNVERIFIED: whether Excel overwrites
rendered constants the original builder writes inside the pivot's own area, which this probe relies on; it needs a
control with stale rendered cells (T-2327).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). The shapes this probe leaves out (several data
fields, two row fields, filters, hidden items, number, date and boolean items, tabular and outline form) are probed
in "GETPIVOTDATA on other pivot shapes (T-2085)" below. Filters are probed in docs/excel-semantics/pivots.md (T-4130, T-4164) and applied
by the engine since T-4165.
Engine (SPEC-0015): the workbook model holds pivot caches and pivot tables (`pivot.rs`, T-2081), read from xlsx by
`crates/rustsheet-xlsx/src/import/pivots.rs` (T-2082). The importer skips, whole and logged, a cache with an OLAP,
external, consolidation or scenario source, a calculated field, no records part
(`saveData="0"`), a bad index, or a value it cannot read (an unknown error name, a `<d>` date before 1900, which
includes time-only dates if Excel writes them so), and a pivot whose `showDataAs` or x14 `pivotShowAs` is unknown,
whose index points past its cache, whose `axis` or `subtotal` is unknown, whose `<filters>` it cannot type or that
passes the filter caps (T-4165, above), which has
no `location` or not one `pivotField` per cache field, or whose parts are reached outside the package (absolute,
escaping or External targets), twice, or through a duplicate `cacheId`: GETPIVOTDATA against it is `#REF!`. A pivot
whose data field is shown as a calculation loads and renders since T-4163; GETPIVOTDATA against it stays `#REF!`
(UNVERIFIED, T-4750: pivots.md "Show-values-as edge cases (T-4161)").
The fuzz target `pivot_import` feeds hostile pivot parts to this importer and GETPIVOTDATA (T-4133, decision 0312).
UNVERIFIED: what Excel answers for each of these (presumably it computes them, refreshing a cache saved without
records); a known gap, not a deviation. A cache `<d>` date is converted to the workbook's date system (T-2393); one before
the 1904 epoch in a `date1904` workbook has no serial there and skips the cache like a pre-1900 date (UNVERIFIED: Excel's answer). The 1904 serial is the 1900 sum minus 1462, so a timestamp's fraction can differ by one ulp from the 1904 sum's; compute the date part in the target system before pivot date items are matched (T-2085). The 1904 values are derived from the 1462-day epoch offset, not from a 1904 pivot workbook.
GETPIVOTDATA (T-2083, `functions/lookup_and_reference/getpivotdata.rs`, `pivot_index.rs`) aggregates the cache records the
pairs pick, in cache order, rather than reading the rendered cell, and passes every cell of probes/getpivotdata-real-pivot.
It answers `#REF!` for SPEC-0015's unknowns, never a guess; T-2085 has probed them ("GETPIVOTDATA on other pivot shapes (T-2085)", owners T-4804, T-4805, T-4806): several data fields, page fields or
hidden items, a filter or AutoShow (`pivot_index.rs` `supported`, T-4165; UNVERIFIED, T-4799: before T-4165 it answered
from the unfiltered records, so a pair naming an item the filter shows was a number and is now `#REF!`), a pair on a pivot with two fields on one axis, a reference meeting two pivots, `""` or an omitted
data_field, a number, date or
boolean as an item (or an item that matches two cache items differing only in case), the same field twice with the same
item, an error as an item (an error as a field passes through, T-4006 section, T-4823), an empty intersection of any aggregate but Sum, a picked record whose data-field value
is not a number, a number or boolean data_field, and an array data_field, pivot_table or field name (an array of items
lifts). It propagates an error in a single-cell pivot_table even inside a pivot, where Excel picks the pivot like
any of its cells (probes/t2085-getpivotdata-items Q!E20:E23; T-4805), and answers `#NUM!` for an overflowing aggregate,
as Excel does (the T-2085 section's "Overflow" paragraph). Count pivots of an all-numeric field match
Excel on gen-m6 8641fa604c5b8e14, b304d8cf05adcc5e and df85bc699c7e87bc (T-2674; they cannot tell COUNT from COUNTA).
Max, Min, Product, Var and StdDev pivots: see the T-2085 section's "Aggregates over one record or none" paragraph.
Structural edits: see "Pivot tables and structural edits" below. xlsx export writes pivots (T-4176; pivots.md "Excel for the web opens engine-exported pivot parts"), except a grouped cache, dropped with its pivots (T-4734).

## GETPIVOTDATA on other pivot shapes (T-2085)
Rule: GETPIVOTDATA answers the value the pivot shows, for every shape SPEC-0015 left unprobed. It reads the shown
value even where the cache disagrees, and an item, total or data field the pivot does not show is `#REF!`. "Shows"
means a cell of the grid. A blank intersection cell, as when a row field crosses a column field, exists and is 0 (DF
B6:J6). A row or column the pivot has no line for is `#REF!`, as with an inner row item missing under its outer item
(LY B6:D6).
Cell ids are sheet Q of each case. The cases are probes/t2085-getpivotdata-data-fields (DF),
-items (IT) and -layouts (LY). The pivots are builder-v2 pivots that Excel renders on open.

**Several data fields.** Pivot M has Sum of Sales, Count of Sales and Sum of Units, with `Values` on the columns.
- A source name shared by two data fields picks the first: `"Sales"` is Sum of Sales, 46 (DF A1).
- A caption picks its data field, ignoring case: `"Count of Sales"` 8, `"sum of units"` with West 23 (DF A3, A12, A13).
- `Values` works as a pair field whose item is the chosen data field's caption: `"Sales","Values","Sum of Sales"` is 46.
  An item naming another data field is `#REF!`, e.g. `"Values","Count of Sales"` or `"Values","Sum of Units"`
  after `"Sales"` (DF A8, A9, A11).
- `"Values"` as data_field is `#REF!` (DF A10).
- A blank cell or `""` as data_field is `#REF!` with several data fields (DF A6, A7, A20).

**Blank and empty data_field.** On S1, a pivot with one data field:
- A blank cell inside the pivot's area gives the grand total: `S1!C3`, where ISBLANK is TRUE, gives 46. This matches
  T-1705's J7, whose blank C3 is inside its pivot.
- A blank cell outside the area is `#REF!`: `Q!Z1` on another sheet, or `S1!Z1` on the pivot's sheet (DF A15, A17).
- `""` is `#REF!`, with or without pairs (DF A14, A16).

**Aggregates over one record or none.** An empty intersection is 0 for every aggregate: CountNums, Product, StdDev,
StdDevp, Var and Varp join Sum, Count, Average, Max and Min (DF C6:J6). One record is `#DIV/0!` for StdDev and Var
and 0 for StdDevp and Varp (DF G5, I5, H5, J5). Totals and intersections of Count, CountNums, Max, Min, Product,
StdDev, StdDevp, Var and Varp over two or more records equal the engine's folds (DF B1:J4, plus the one-record
cells of Count, CountNums, Max, Min and Product).

**Overflow.** A Sum or Product past the double range is `#NUM!`: 1E308+1E308, and 1E200·1E200 (DF K1:K2, L1:L3).
A total that fits is the value: West Sum 2E200 (DF K3).

**Number, date and boolean items** (IT).
- Number field: an item matches by value, given as a number (`1`, `1.0`, `2.5`, `10`), as numeric text (`"1"`,
  `"2.5"`, `"10"`) or as a reference to a number. `TRUE` is `#REF!` (A1:A9).
- Date field, shown m/d/yyyy: a serial, `DATE(2024,1,1)`, a reference, or the date as en-US text (`"1/1/2024"`)
  each match. Other forms are `#REF!`: ISO text `"2024-01-01"`, the serial as text `"45292"`, and a serial with a
  fraction, 45292.5 (B1:B7).
- Boolean field: `TRUE`, `FALSE`, `"TRUE"` and `"true"` match. `1` is `#REF!` (C1:C5).
- Non-ASCII case folding: `"été"` and `"ÉTÉ"` both match the item `Été`, and likewise `Øst` and `Ärger`. The
  data_field `"SUM OF SALES"` and the field `"CODE"` also match (D1:D8).

**A repeated field.** The same field twice with the same item is that item's value, also when the field and item
are written in different case: `"Region","East","region","EAST"` is 21 (IT E1, E2).

**Records that are not numbers.** A Sum over a text record, here `"abc"`, ignores it: East is 5. It also ignores a
blank: Nord is 3. An error record is the error: Mid is `#N/A`, and so is the grand total (IT E13:E17).

**Errors in the arguments.** An error as a field name is that error (`NA()` gives `#N/A`, IT E3). An error as an
item is `#REF!` (E4, E5). A number, boolean or referenced number as data_field is `#REF!` (E6:E8). An error held by
the pivot cell given as pivot_table is not passed through: the cell only picks the pivot, like any cell of it. So
`GETPIVOTDATA("Val",E!B6,"Region","East")` with E!B6 holding `#N/A` is 5, the same as with the non-error cell E!A4,
and with West it is 7. A range holding the error cell answers the same way (E20:E23). Without pairs both give `#N/A`
(E11, E12), which is presumably the pivot's grand total (itself `#N/A` here) by the same rule. These cells cannot tell
that apart from the error being passed through, so the no-pair form is UNVERIFIED. The engine passes the cell's error through today (T-1703's
rule for a pivot_table with no pivot), which is wrong inside a pivot.

**Arrays.** An array data_field lifts: `{"Sales","Units"}` spills 46 and `#REF!`, since Units is not a data field
(IT E9:F9). So does an array field name: `{"Region","Product"},"A"` spills `#REF!` and 30 (E10:F10).

**Page fields and hidden items** (LY).
- Page field Region set to East: the grand total is the East total, 21. A pair naming the page field with its
  selected item is 21, with another item `#REF!`. Inner pairs answer within East: Product A is 14 (A1:A4).
- A pivot_table reference into the page-field area above the location, `PF!A1` or `PF!B1`, is that pivot
  (A5, A6).
- A page field at (All): the grand total is 46, and a pair on the page field is `#REF!` (A7, A8).
- A hidden row item C: the totals leave it out (grand total 44, East 19), and the pair `"Product","C"` is
  `#REF!` (A9:A12).

**A range over two pivots.** TW holds Left (rows Region, at A3) and Right (rows Product, at F3), and
`TW!$A$1:$J$12` covers both. The committed fill that asked it (commit 82ab72cda of this PR's history, produced
2026-11-07T07:29:43) answered from Left: the grand total 46 (both pivots total 46), Region East 21, Product A
`#REF!`. A later fill of an edited workbook, not committed, answered from Right instead: Region East `#REF!`,
Product A 30. That one observation is unrecorded, so whether the pick is stable, and what decides it, is not
established. The sidecar's `pivots` order (the saved parts' order) does not explain it: the committed fill lists
Right first, yet Left answered. The case no longer asks it. A cell of one pivot (`TW!$F$3`) picks that pivot (A15).
Status: UNVERIFIED which pivot a range over two pivots picks; a probe that records several fills is needed. The
engine's `#REF!` stays.

**Two row fields** Region and Product, the same in compact, tabular and outline form. The TB and OL pivots set
compact and outline on the table and on every pivotField, and Excel renders them as such: area A3:C11 with the
data in column C, and "East Total" lines in TB. CP renders in compact form, A3:B11.
- The grand total is 46, a Region subtotal 21 and an inner item 14. Both pairs in either order work: Product A
  with West is 16 (LY B1:D5).
- The inner field alone, `"Product","A"`, is `#REF!`. It is not the Product A total, which the pivot does not show
  (B4:D4).
- An inner item with no records under its outer item (West/C) is `#REF!`, not 0, because the pivot shows no such
  row (B6:D6).
- With the outer field's subtotals off, the Region subtotal is `#REF!` and inner items still answer (E1:E6).

**Summation order.** The answer is the shown cell, so it follows pivots.md "Summation order of Sum, Average and
Product totals", not record order. That rule adds the leaf cells in display order, here first-seen: East/B, then
East/A (SO!A5:A6). East is 0.3 + 0.30000000000000004 (A's cell, 0.2 + 0.1) = 0.6000000000000001. The grand total adds
the leaf cells East/B 0.3, East/A 0.30000000000000004 and West/A 0.7, which gives 1.3. Record order gives
1.2999999999999998 (LY F1:F5). Adding the subtotals East + West would also give 1.3 here, so this probe does not
tell the two apart; the leaf rule is pivots.md's evidence.

**A saved render.** On the T-4586 pivot (refreshOnLoad off, saved cells East 99 and Grand Total 99, no West row),
GETPIVOTDATA reads the saved cells: the grand total and East are 99, while West, which has records in the cache but
no row, is `#REF!` (LY H1:H5). A render from the cache would give East 5, and a refresh 15.

**Calculated fields.** A calculated field answers by its name or by its caption: `"Double"` is 92 and 42, and
`"Sum of Double"` with West is 50 (LY G1:G3).

Not probed:
- An array as pivot_table (no plain formula makes an array of references to one pivot).
- Two cache items differing only in case: Excel for the web calls such a file corrupt (pivots.md "Case-variant text
  items make Excel for the web call the file corrupt").
- Grouped fields: GETPIVOTDATA on a group label is T-4228's ("Grouped pivot fields import as their group items (T-4225)"
  below).
- Calculated items: Excel for the web refuses them.
- A cache saved without records (`saveData="0"`, SPEC-0015 unknown 12). The case builder writes every cache with its
  records and marks it refreshOnLoad, and a saved, non-refreshing cache with no records part is not something it
  can write. The importer skips such a cache, so the engine answers `#REF!` (the import list above).

Evidence: probes/t2085-getpivotdata-data-fields, -items and -layouts (excel-graph, 2026-11-07, T-2085).
Status: VERIFIED (excel-graph).
Ours: the T-2083 engine answers `#REF!` for most of these, or a value where Excel has `#REF!` (a blank cell outside
the pivot). Implementing them is T-4804 (data fields and aggregates), T-4805 (items, records, arguments and arrays)
and T-4806 (shown values and shapes). The calculated-field cells are T-4174's, and the date items' shown text is
T-4807's.

## GETPIVOTDATA error arguments (T-4006)
Rule: against a live compact-form pivot, an error argument of GETPIVOTDATA gives:
- `data_field` typed as an error literal: that error (`#N/A,Pivot!$H$3` is `#N/A`; `#NULL!,Pivot!$H$3,2,"C"` is
  `#NULL!`). The same error read from a cell is **`#REF!`**, a miss like an unknown name
  (`Data!G9,Pivot!$H$3,"Product",Data!G1` with Data!G9 `#DIV/0!` is `#REF!`).
- `field`, typed or read from a cell: that error (`"Units",Pivot!$H$3,#NAME?,2` is `#NAME?`;
  `Data!G1,Pivot!$H$3,Data!G9,"C"` is `#DIV/0!`).
- `item`, typed or read from a cell: `#REF!`, like any item not in the field
  (`"Units",Pivot!$H$3,"Product",#DIV/0!`).
- Order: a field error beats an item error (`#REF!,#NAME?` is `#REF!`; `Data!G9,Data!G9` is `#DIV/0!`) and a
  referenced data_field error (`Data!G9,Pivot!$H$3,Data!G9,"A"` is `#DIV/0!`); a typed data_field error beats an
  item error (`#NULL!,Pivot!$H$3,"Product",#VALUE!` is `#NULL!`). A typed data_field error against a field error
  is unseparated (both `#NULL!` in the one cell that has both).
- The other non-text `data_field` classes are `#REF!` with no other miss in the call: 0, -1, 0.5, `TRUE`, `"2.5"`
  and the 2 x 2 range `Data!A1:B2` (`GETPIVOTDATA(0,Pivot!$H$3)`), and 0, -2.5, 1.5, `"12"`, `FALSE` read from a
  cell or the range `Data!C3:D4`, against the whole pivot with a valid pair (one `#REF!` each, no spill seen).
Evidence: gen-m6 16b67941f34e0346, ccc550562ee5fd06, 37eb2ebaf9386ade and 2963992505456e99 (excel-graph,
2026-11-07, T-4006; `--policy arg-pairs`). Each case's Calc!A1 is a valid control,
`GETPIVOTDATA("Units",Pivot!$H$3,"Product","A")`, which Excel answers 6.75, so the pivot was live. Cells: typed
data_field 16b67941f34e0346 Calc!A34, A562; referenced data_field 37eb2ebaf9386ade Calc!A43 and 16b67941f34e0346
Calc!A37; field 2963992505456e99 Calc!A175 and 16b67941f34e0346 Calc!A97; item 37eb2ebaf9386ade Calc!A298; order
2963992505456e99 Calc!A223, A226, ccc550562ee5fd06 Calc!A16, A13 and 37eb2ebaf9386ade Calc!A88; other classes
16b67941f34e0346 Calc!A4 to A43.
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). Ours: all rules implemented (T-4823; a computed
error such as NA() as data_field is unprobed and treated as typed).

## GETPIVOTDATA on a recordless item of an ungrouped pivot field (T-4248)
Probed (probes/t4248-getpivotdata-recordless-item, excel-graph): the cache lists "North" as a shared item no record uses
(`refreshOnLoad` off; UNVERIFIED: that Excel drops the item on a refresh, and the `missingItemsLimit` link), over Data East 10, West 20, East 5.
- `showAll="0"`: the item is not shown and `GETPIVOTDATA("Sales",pivot,"Region","North")` is `#REF!`, as for a group with no
  record (see below) and for a text absent from the cache ("Nowhere").
- `showAll="1"`: the item is shown with a blank value (UNVERIFIED: the layout is the builder's, no read-back cell) and the same call is `0` (like an empty intersection,
  probes/getpivotdata-empty-intersection); the item text matches case-insensitively (`"north"` is 0).
- Control: the grand total (35) and a record item (East, 15) answer as usual in both.

The same rule holds on a column field and on a grouped row field (probes/t4311-pivot-showall-column-and-groups,
excel-graph, 2026-11-05, T-4311):
- Column field Product with "C" an unused shared item, over East/A 10, West/B 20, East/B 5: with `showAll="0"`
  `GETPIVOTDATA("Sales",pivot,"Product","C")`, the same with `"Region","East"` and with `"c"` are `#REF!` (Calc A1, A3,
  A5); with `showAll="1"` all three are 0 (B1, B3, B5). "Nowhere" is `#REF!` and the controls (B 25, East/B 5, total
  35) answer as usual in both.
- A row field grouped by 0.1 from 0 to 0.4 over N = 0.05, 0.25, 0.27, 0.35 (S 1, 2, 4, 8), so `<0`, `0.1-0.2` and
  `>0.4` hold no record: with `showAll="0"` Excel lays out `0-0.1`, `0.2-0.3`, `0.3-0.4` and the total, and those
  three labels are `#REF!` (Calc D1, D3, D6; G1:H5); with `showAll="1"` it lays out all six groups, the empty ones
  with a blank value (Calc I1:J8, read back from Excel's own layout), and the three labels are 0 (E1, E3, E6).
Engine (T-4276): the importer keeps every `<sharedItems>` entry, used or not, and each `<pivotField>`'s `showAll` (on
when absent, ECMA-376's default; Excel writes `showAll="0"`). GETPIVOTDATA on an ungrouped field's item that no record
holds is `#REF!` when its field's `showAll` is off and 0 when on (`shown_group`, pivot_index.rs), on the row and the
column axis and on a grouped field alike (T-4311), and the engine's render (`pivot::compute` under `showAll`, `pivot::render`; T-4149) shows a grouped field's empty groups with a blank
value under `showAll`. The eval adapter
appends a spec's `unused_items` to the row field's items after the records' own and takes its `show_all`.

## Grouped pivot fields import as their group items (T-4225)
Observed (Excel-authored caches, docProps/app.xml Microsoft Excel 16.0300): Excel stores no record values for a grouped
field. A `<cacheField>` grouped in place carries `<fieldGroup par base>` after its `<sharedItems>`; a grouping field Excel
adds (`Quarters`, `Years`, `Hours`, `Days`) is a `<cacheField databaseField="0">` written after every database field,
with `<fieldGroup base>` naming the database field it groups; the records part holds the database fields only. Each group
lists its labels in `<groupItems>` (`<s>` items), and a pivot field's `<item x>` indexes those labels, not the base
field's items. The date layouts are fixed: a first item for values below `startDate` (`<1/1/2019`), the calendar part, and
a last item for values past `endDate` (`>1/14/2019`):
- `groupBy="seconds"` and `"minutes"` 62 items each, `:00` to `:59` (seconds: Excel's refresh of
  probes/t4269-pivot-time-group-number-items renders `:00`, `:03`, `:05` ...); `hours` 26 items `12 AM` to `11 PM`; `days` 368 items `1-Jan` to `31-Dec`
  with `29-Feb` always present (corpus-bench/52379 fields `Date/Time`, `Hours`, `Days`);
- `months` 14 items `Jan` to `Dec`; `quarters` 6 items `Qtr1` to `Qtr4`; `years` one item per year from `startDate`'s
  to `endDate`'s, 123 items for 1899-12-30 to 2020-03-28 (`1900` to `2020`; corpus-bench/59932 fields `Date`,
  `Quarters`, `Years`).
- Quirk: 59932's `Date` field, which holds blanks (`containsBlank="1"`, shared item 5767 is `<m/>` and records point at
  it) and starts at serial 0, labels its first month item `(blank)` where its `Quarters` and `Years` groups of the same
  field and range label it `<1/0/1900`.
- A grouped field has no blank item of its own: 59932's pivot lists exactly 14, 6 and 123 `<item x>` for `Date`,
  `Quarters` and `Years`, while an ungrouped field with a blank lists it as an extra item (52379 `User`: 58 `<s>` and
  `<m/>`, 59 `<item x>`). So a blank record falls in one of the group items.
Engine: the importer turns every group into an ordinary cache field whose items are the group labels (text) and whose
records are each record's group (`crates/rustsheet-xlsx/src/import/pivots/groups.rs`), so GETPIVOTDATA addresses a
grouped field by its label (`"Quarters","Qtr1"`). Numbers (`rangePr startNum endNum groupInterval`) follow the probed
rule below (`NumberGroups`, crates/rustsheet-engine/src/pivot/number_groups.rs, T-4235). For dates the record-to-group
rule is ECMA-376's layout read off these caches: below `start` is item 0, past `end` the last item; otherwise dates fall
in their second, minute, hour (the time rounded to the nearest second),
day of a leap year, month, quarter or year counted from `startDate`'s year (the day and time split as YEAR and HOUR,
MINUTE, SECOND split a serial: `carried_day`, `time_of_day_parts`), and a `<discretePr>` list maps base shared item i
to its i-th `<x v>`. A blank base value goes in item 0 (the `(blank)` label above; UNVERIFIED, see the date-group
bullets: a refresh drops such a grouping, so no probe can show it). The importer skips the cache
(logged) for a day group of more than one day, a date group in a 1904 workbook, an item count other than the layout's
(checked when the grouping is read, so even over a base field that holds no date; since T-4246),
a range group over text, a database field after a grouping field, and a base that is not a database field. A source
field's bare `<fieldGroup par>` beside a manual group (probes/t4170-pivot-manual-groups) only names the field Excel
added, so it groups nothing (accepted only when `par` names an added field whose `base` is that field); each added field keeps its base, and a refresh derives it again (T-4171; pivots.md
"Manual groups and date levels as Excel for the web renders them").
Numeric range groups (`<rangePr startNum endNum groupInterval>`), probed (T-4228):
- Rule: a value below `startNum` goes in the first item and one past `endNum` in the last. A value equal to `endNum`
  goes in the last inner group (1 in `0.9-1` by 0.1 from 0 to 1; 9 in `6-9` by 3 from 0 to 9). Any other value goes in
  inner step `floor(q)` (from 0), where q = (v - startNum) / groupInterval **rounded to 15 significant digits**: 0.3, 0.6
  and 0.7 by 0.1 land in `0.3-0.4`, `0.6-0.7` and `0.7-0.8` (q is 2.9999999999999996, 5.999999999999999,
  6.999999999999999), 0.2999999999999999 (q 2.9999999999999987) too, while 0.299999999999999 (q 2.99999999999999)
  stays in `0.2-0.3`; by 0.001, 0.12299999999999961 (q 4e-13 below 123) is in `0.123-0.124` and 0.12299999999999921
  (8e-13 below) in `0.122-0.123`, so the rounding is relative, not an absolute tolerance on q.
- Quirk: a step that rounds past the last inner group goes in the **last item** (`>end`), not the last inner group, even
  though the value is below `endNum`: 0.9999999999999999 by 0.1 from 0 to 1 is in `>1` while 1 is in `0.9-1`;
  1.9999999999999998 (from 1 to 2) is in `>2`, 8.999999999999998 (by 3 from 0 to 9) in `>9`.
- A group with no record is not shown (`showAll="0"`) and GETPIVOTDATA on its label is `#REF!`. With `showAll="1"` every
  group is shown, the `<start` and `>end` ones too, each empty one with a blank value, and GETPIVOTDATA on an empty
  group's label is 0 (probes/t4311-pivot-showall-column-and-groups, below).
- GETPIVOTDATA on the grouped field: a text item is the label (`"0.3-0.4"`), case-folded for the field name (`"n1"`),
  not trimmed (`" 0.3-0.4"` is `#REF!`) and not read as numbers (`"0.30-0.40"`, `"0.3"` are `#REF!`); a reference to a
  rendered label cell works, and an array of labels lifts. A **number** item picks the group whose bound equals it,
  within the same rounding (0, 0.2, 0.6, 0.6000000000000001, 0.7, 0.7000000000000001 and 0.9 by 0.1; 1.1, 1.3,
  1.3000000000000003 from 1; 3 by 3): an inner group's lower bound, `start - interval` for the `<start` item and the
  `>` label's bound for the `>end` item (1 gives the `>1` total). A number inside a group but not its lower bound (0.35,
  4), TRUE, an omitted item and `"(blank)"` are `#REF!`. `"Sum of S1"` works as data_field as on an ungrouped pivot.
- Quirk (T-4249): GETPIVOTDATA's number rule is truncation toward zero. With q the 15-digit quotient and k = q truncated
  toward zero, the number matches item k when q is not above k: k = -1 is `<start`, 0 to n - 1 the inner groups and n
  `>end`; anything else is `#REF!`. So a positive q must be whole, but a negative one truncates up: by 3 from 0, -1, -2,
  -1.5 and -0.000001 give the `0-3` total, -2.9999999999999996, -3, -3.5 and -4.5 the `<0` total, and -6 is `#REF!`
  (and -0.5 by 0.1 from 0 is `#REF!`, q -5). A number past `endNum` is `#REF!` unless it is the `>` bound: from 0 to
  10 by 3, 12 is `>12` and 10, 11 and 15 are `#REF!`, 9 and 9.000000000000002 are `9-12`; from 0 to 9, 9 and
  8.999999999999998 (q rounds to 3) are `>9`, 8.9999999999999 is `#REF!`.
- Quirk: Excel writes its **own labels whatever the `<groupItems>` list says**, of any length (one, two too long, one
  or two short) and of the layout's own length too: a list of five wrong texts for 0 to 9 by 3 shows `<0`, `0-3`,
  `3-6`, `6-9`, `>9`, and a right-length list with `>10` for 0 to 10 by 3 shows `>12`. GETPIVOTDATA on a file label
  that is not one of Excel's is `#REF!`. (T-4228 saw it first as a list one item too long, 6 items for 0 to 9 by 3,
  shown as `<0`, `>9`, `0-3`, `3-6`, `6-9`.)
- Layout (T-4249): a span that is not a whole number of intervals gets `ceil` inner groups and the `>` label names the
  last inner group's upper bound, not `endNum`, while records still split at `endNum`: 0 to 10 by 3 shows `<0`, `0-3`,
  `3-6`, `6-9`, `9-12`, `>12`, with 10 in `9-12` and 10.000000000000002, 11 and 12 in `>12`; 0 to 9 by 2.5 ends
  `7.5-10`, `>10`.
- Whole-number labels (T-4249, T-4287): when every value of the base field, `startNum`, `endNum` and `groupInterval`
  are whole numbers,
  an inner group reads `lo-(lo + interval - 1)` and the last one ends at `endNum` when that is larger: 0 to 9 by 3
  over whole data is `0-2`, `3-5`, `6-9`; 0 to 10 by 3 is `0-2`, `3-5`, `6-8`, `9-11`, `>12`. One value that is not
  whole (2.5, 8.999999999999998) gives the `lo-(lo + interval)` labels below; so do whole data by 2.5 (`0-2.5`, ...)
  or from 0.5 by 3 (`<0.5`, `0.5-3.5`, ..., `>9.5`), and whole data with an `endNum` that is not whole (T-4287): 0 to 9.5
  by 3 is `0-3`, `3-6`, `6-9`, `9-12`, `>12` (9 in `9-12`; 10, 11, 12 in `>12`), 0 to 8.5 and 0 to 7.5 by 3 end `6-9`,
  `>9` (records still split at `endNum`: 8 is in `>9` for 0 to 7.5).
- Quirk (T-4287): **a numeric grouping over a blank, or over date-formatted cells among numbers, is dropped** on refresh,
  as a date grouping over a blank is (below): whole data 0 to 9 by 3 with one blank record shows `-1`, `0`, `3`,
  `(blank)`, `6`, `9`, `10` (record order), and 0 to 9 by 3 over -1, 1, 4, 7, 10 with 1 and 4 formatted `yyyy-mm-dd`
  shows each value (`4.5` too) as its own item. **Over date cells alone it is dropped too** (T-4316): 0 to 9 by 3 over
  1, 2, 4, 7, 10, all `yyyy-mm-dd`, shows `1`, `2`, `4`, `7`, `10` and GETPIVOTDATA on `0-2` ... `>9` is `#REF!`; so is
  1, 2, 4.5, 7, 10 (probes/t4316-pivot-group-dates-alone, t4316-pivot-group-dates-alone-frac). So any date cell drops it.
- Relabel order (T-4253, T-4249): the file's items whose text is one of Excel's labels come first, in the file's order,
  then Excel's other labels in layout order. Excel's five labels for 0 to 9 by 3 listed as `<0`, `3-6`, `0-3`, `>9`,
  `6-9` keep that order; a list one short (`<0`, `0-3`, `3-6`, `>9` over whole data, whose labels are `0-2` ...) shows
  `<0`, `>9`, `0-2`, `3-5`, `6-9`; two too long (`<0`, `0-2`, `3-5`, `6-8`, `9-11`, `12-14`, `>9`) shows `<0`, `0-2`,
  `3-5`, `>9`, `6-9`. Pivot 3 lists `<0`, `>9` first because the file has both; with 13 items for
  the 12 of 0 to 1 by 0.1, all wrong text (`w0`, ...), the order is the layout's; with only `>1` (last) it is `>1`,
  `<0`, `0-0.1`, ...; with only `0.5-0.6` (item 3), `0.5-0.6`, `<0`, `0-0.1`, ... (not the file position); with `>1` first and `<0` last, `>1`, `<0`, ...; with `0.7-0.8` (item 2) and `0.2-0.3` (item 5),
  `0.7-0.8`, `0.2-0.3`, `<0`, `0-0.1`, ...; `<0` alone, first or at item 11, and Excel's 12 labels plus one extra keep
  the layout order. The match is on the whole text, ignoring case but not spacing (T-4287): `>3e+20`, `2e+20-3e+20`, ... match Excel's
  `>3E+20`, ... and keep the file's order (Excel still writes `E`), while `6 - 9`, ` 3-5` and `3-5 ` match nothing and
  come after the matched items.
- Quirk (T-4316): **a label the file lists twice takes its last place**: `6-9`, `3-5`, `6-9`, `w3`, `w4` for 0 to 9 by 3
  over whole data shows `3-5`, `6-9`, `<0`, `0-2`, `>9` (`<0`, `<0`, `0-2`, ... and five `x` keep the layout order).
  A **non-text item** (`<n v="0"/>`) matches nothing and is skipped: `<0`, 0, `3-5`, `6-9`, `>9` shows `<0`, `3-5`,
  `6-9`, `>9`, `0-2`; 3, `6-9`, `3-5`, ... shows `6-9`, `3-5`, `<0`, ...; five numbers -1, 0, 3, 6, 9 the layout order
  (probes/t4316-pivot-group-label-twice, t4316-pivot-group-number-item). GETPIVOTDATA on each label works as usual.
- Relabel text (T-4253): `<start`, `bound(k)-bound(k+1)` and `>` with the last bound (data not all whole), each bound start + k * interval rounded to 15
  significant digits and printed in plain decimal, not General: by 1e-5 the labels are `0-0.00001` ... `>0.0001` (General
  would print `1E-05`), by 1/3 `0-0.333333333333333`, `0.333333333333333-0.666666666666667`, `0.666666666666667-1`, by
  0.1 from 1.1 `<1.1`, `1.1-1.2`, ..., `2-2.1`, `>2.1`, by 0.25 from -1 `<-1`, `-1--0.75`, ..., `-0.25-0`, `0-0.25`, by
  12345678901.2 `49382715604.8-61728394506`, as text coercion writes each number (numbers.md "Number → text
  conversion", `number_to_text`). T-4287 agrees past those points: by 1e15 the bounds are plain (`0-1000000000000000`,
  `>4000000000000000`), by 1e20 they are `0-1E+20`, ..., `>3E+20` (whole data too, where `lo + interval - 1` is
  `lo + interval` in binary), by 1e-6 and 1e-15 plain (`0.000001-0.000002`, `0.000000000000001-0.000000000000002`), and
  from 100000000000000.5 by 1 (each bound a 15-digit tie) `<100000000000000`, `100000000000000-100000000000001`, ...,
  `>100000000000003`: each tie writes its lower neighbour.
- excel-graph lays a grouped pivot out itself on load (the case builder writes no rendered cells for one, and the
  probe's `=P!A3&""` reads show Excel's layout), so these are Excel's own groupings, not the file's.
Evidence: probes/t4228-pivot-range-groups (excel-graph, 2026-11-04, T-4228): five pivots whose S values are distinct
powers of two, so each group's sum names its records; probes/t4253-pivot-decimal-relabel and
probes/t4253-pivot-relabel-order (excel-graph, 2026-11-05, T-4253): eight relabeled decimal groupings each and the
rendered labels; probes/t4249-pivot-range-group-edges and probes/t4249-pivot-range-group-labels (excel-graph,
2026-11-05, T-4249): thirteen pivots over wrong lists, partial spans and whole data, and GETPIVOTDATA number items
past `endNum` and below `startNum`; probes/t4287-pivot-group-label-match, -bound-text, -whole-end and
-dates-as-numbers (excel-graph, 2026-11-05, T-4287): sixteen pivots on label matching, large, tiny and tie bounds,
fractional ends, a blank and date cells; probes/t4316-pivot-group-label-twice, -number-item, -dates-alone and
-dates-alone-frac (excel-graph, 2026-11-05, T-4316): eight pivots on repeated and number items and date cells alone.
Status: VERIFIED (excel-graph). Engine (T-4235, T-4253, T-4249): `NumberGroups`
groups by the rule, picks a number item by the truncation rule, and always writes its own labels in the relabel order
above (`NumberGroups::new`, `labels`, `lower_bound_index`). T-4287: whole labels need a whole `endNum`
(`NumberGroups::new`), the label match folds case (`fold_case`), and `Txn::add_pivot_table` leaves the field ungrouped
over a blank or any date cell (`grouped`, `holds_dates`; `render` writes a number item as its value);
the importer keeps a saved grouping, as for dates. T-4316: a repeated label takes its last place and a number item
is skipped (`NumberGroups::new`), and any date cell, not only dates among numbers, drops the grouping. A group with no record
is `#REF!` while its field's `showAll` is off and 0 while it is on (T-4311). The importer and
the eval adapter share it (`group_by_numbers`); the adapter has the engine lay a grouped spec pivot out (captions, the
groups that hold a record, grand total). The 15-digit rounding of q, not of v, is a structural model the fresh sweep
probes/t4235-pivot-range-rounding confirms: 105 of 105 points, against 99 for rounding v first and 87 for the plain
quotient (T-4235 Outcome).
Date groups (`<rangePr groupBy startDate endDate>`), probed (T-4236):
- Rule: the cache rounds a date to the nearest second, and the rounded date is what is compared to `startDate` and
  `endDate` and split into its calendar part (as YEAR, MONTH and HOUR split it: 23:59:59.5 or later is the next day).
  2019-01-31 23:59:59.4 is in `Jan` and 23:59:59.5 and .9 in `Feb`; 2018-12-31 23:59:59.5 is in `Jan` of a group
  starting 2019-01-01, not `<1/1/2019`; 2019-03-31 23:59:59.5 is `Qtr2`, 2017-12-31 23:59:59.5 is `2018`, 10:59:59.5 is
  `11 AM` and 23:59:59.5 `12 AM`. A date equal to `endDate` is inside the range (2020-01-01 in `Jan` of a group ending
  then; 2019-01-02 00:00 in `12 AM` of an hour group ending then), and anything later, even the same day (12:00 on
  `endDate`), is in the last item. Ungrouped, two dates within a second of each other are one cache item too.
- Serials below 61 group by the 1900 calendar: 0.5 is below a group starting at serial 1 (`<1/1/1900`); 1 and 31 are
  `Jan`; 32, 59, 59 at 23:59:59.6, 60 and 60.75 are `Feb`; 61 is `Mar`. Day groups count the day of a leap year: 60 and
  60.5 are `29-Feb`, 61 `1-Mar`, 366 `31-Dec`, 367 `1-Jan`, and 2019-02-28 and 2019-03-01 are `28-Feb` and `1-Mar` (no
  2019 date is in `29-Feb`).
- Quirk: on refresh, a date grouping of a field that holds a blank source cell is dropped. Every pivot of
  probes/t4236-pivot-date-groups with a blank renders each (second-rounded) date as its own item and the blank as
  `(blank)`; GETPIVOTDATA then takes `"(blank)"` or an omitted item for the blank and a date serial
  (`DATE(2019,1,1)`) for a date item, while every group label is `#REF!`. The ungrouped items are listed in record
  (first-appearance) order, not sorted, with `(blank)` at its record's place (pivot M: `... 43876, (blank), 43631`;
  pivot H: `... 43467.25, (blank), 43466.5`). Which group a blank takes in a cache that
  keeps both (59932) is therefore not probeable.
- GETPIVOTDATA on a date-grouped field: a text item is the label, case-folded (`"jan"`, `"JAN"`, `"12 am"`, `"qtr1"`,
  `"29-feb"`, `"1-mar"`); the `<1/1/2019` and `>1/1/2020` labels work while their group has a record. A **number** item
  picks the group whose calendar value equals its integer part: the hour (`0` and `TIME(10,0,0)` are `12 AM`, `10` is
  `10 AM`), the leap-year day (`1` is `1-Jan`, `60` `29-Feb`, `61` `1-Mar`), the month (`1` is `Jan`; `31`, `32`, `60`
  are `#REF!`), the quarter (`1`) and the year (`2018`, `2018.5`). A date serial (`DATE(2019,1,1)`, `43525`,
  `DATE(2018,1,1)`, `DATE(2019,1,1)+TIME(10,0,0)`), a text number (`"1"`, `"10"`, `"2,018"`), another spelling (`"Q1"`,
  `"January"`, `"01-Mar"`, `"2/29/1900"`), `TRUE`, `"(blank)"` and an omitted item are `#REF!`.
Evidence: probes/t4236-pivot-date-groups and probes/t4236-pivot-date-groups-no-blank (excel-graph, 2026-11-04,
T-4236): pivots grouped by months, hours, quarters, years and days whose S values are distinct powers of two; the
second case is the first with every blank replaced by a date. Status: VERIFIED (excel-graph). Engine (T-4246):
`DateGroups` groups a record by the rule and picks a number item by its calendar value, shared by the importer, the
eval adapter and GETPIVOTDATA; a group with no record is `#REF!`, as on a numeric range field (`showAll="0"`; with
`showAll="1"` the numeric range field's is 0, T-4311, and a date group's is taken to follow it, unprobed).
- A number item on a seconds or minutes group is the second or minute, 0 to 59, by the hour rule: `0`, `5`, `30`,
  `59`, `59.5` and `59.999` pick `:00`, `:05`, `:30` and `:59`; `60`, `61` and `DATE(2019,1,1)` are `#REF!`. The
  integer part is **truncated toward zero**, not floored, on all three time groups: `-0.5`, `-0.0001`, `0.5`,
  `TIME(0,0,5)` and `TIME(0,1,0)` are `:00` (seconds and minutes) and `12 AM` (hours), `1.5` is `:01`, while `-1` is
  `#REF!`. Text items match the label (`":05"`, `":00"`; `":00"` on an hour group, `"5"` and `"-0.5"` are `#REF!`).
  Evidence: probes/t4269-pivot-time-group-number-items (excel-graph, 2026-11-05, T-4269). Status: VERIFIED
  (excel-graph); the engine (`DateGroups::number_item`) matches every cell.
- Quirk: the cache's rounding to the second divides the serial by the double 1/86400 and rounds half up:
  `floor(x / (1/86400) + 0.5)` seconds, not `x * 86400` and not HOUR's rule. A date a hair below a half second can
  therefore round up. Serial 43466.458327546294 (written 10:59:59.5, but 2.2e-7 s below the half) is in `11 AM`
  (no-blank pivot H, Calc E45 and F13), while HOUR and SECOND of it give 10 and 59. 25006.524265046293 (2.6e-7 s
  below 12:34:56.5) is 12:34:57, though its double product with 86400 is ...96.4999995. A 1900 date 2.2e-7 s or
  1e-8 s below the half stays down, because a small serial keeps that much precision. Of five candidates (ms then s,
  15 significant digits, the fraction's product, the engine's HOUR split, the serial's product), the serial's
  product fit the 36 points of probes/t4261-pivot-half-second. The first fresh sweep rejected it (47 of 48), and
  dividing by 1/86400, which fits all 84, was frozen. A second fresh sweep confirmed it at 48 of 48 (32 of them
  points where the two disagree; the product gets 14), and an unbiased log-uniform third sweep at 48 of 48 (the
  product 44, exact correct rounding 27). The R4 record, with its spent first hold-out, is in the T-4261 Outcome.
  UNVERIFIED: a `startDate` or `endDate` within half a second of such a point, which uses the same rule.
Evidence: probes/t4261-pivot-half-second, t4261-pivot-half-second-fresh, -fresh-2 and -fresh-3
(excel-graph, 2026-11-05, T-4261): dates a delta of 1e-9 to 0.4 s either side of a half second, from 1900 to 9998,
in an ungrouped pivot (a blank drops the grouping), whose rendered labels show each record's rounded second.
Status: VERIFIED (excel-graph). Engine: `split_serial` (pivot/date_groups.rs), shared by the groups,
`ungroup_dates`, the importer and the `startDate` and `endDate` comparisons.
Engine (T-4260): `Txn::add_pivot_table` drops a date grouping over a blank as the refresh does (`ungroup_dates`:
second-rounded `Date` items in record order, by the same `split_serial` rule as the groups), the layout writes a date
label as its serial and the blank as `(blank)` (`item_cell`, pivot/layout/render.rs), and GETPIVOTDATA picks an ungrouped date item by an
equal serial and the blank by `"(blank)"` (`pivot_number_item`, `pivot_blank_text`). A serial within the second of
an item but not equal to it, `"(BLANK)"` in another case, and a field that also holds the text `(blank)` are
UNVERIFIED. The importer keeps a saved grouping (59932).
Implemented in: crates/rustsheet-xlsx/src/import/pivots/groups.rs (`apply_groups`), import/pivots.rs (`read_definition`),
crates/rustsheet-engine/src/pivot/number_groups.rs (`NumberGroups`, `group_by_numbers`), pivot/date_groups.rs
(`DateGroups`, `group_by_dates`), pivot/layout/render.rs (`item_cell`, T-4149),
functions/lookup_and_reference/pivot_index.rs (`pivot_number_item`, `shown_group`, `pivot_blank_text`).
Tasks: T-4225, T-4228, T-4235, T-4236, T-4246, T-4249, T-4253, T-4260, T-4261, T-4287, T-4316
Manual (discrete) groups and several date levels on one field: pivots.md "Manual groups and date levels as Excel for
the web renders them" and "GETPIVOTDATA on manual groups and date levels (T-4170)" below.

## GETPIVOTDATA on manual groups and date levels (T-4170)
Rule: GETPIVOTDATA addresses a discrete group's added field and each date level by its field name and label, as the
M6 groups above, on the pivots of pivots.md "Manual groups and date levels as Excel for the web renders them" (S
values distinct powers of two, so each total names its records).
- Manual groups (probes/t4170-pivot-manual-groups Calc, against MC: rows Product2, Product): a group label is its
  total (`"Product2","Group2"` 67, `"Group1"` 140, the group of one `"North"` 16), case-folded (`"group1"` 140) but
  not respaced (`"Group 1"` is `#REF!`); an ungrouped item is an item of the group field (`"Product2","F"` 32); a
  source item is not (`"Product2","A"` is `#REF!`). The inner field alone works for an item that sits under one
  group (`"Product","A"` 65, `"Product","E"` 16); a pair works where the item is in the group (`"Group2"`, `"A"` 65;
  `"F"`, `"F"` 32) and is `#REF!` where it is not (`"Group1"`, `"A"`); the grand total is 255. On the tabular MT
  `"Product2","Group1"` is 140 (Calc!B14), and on MG (Product2 alone) `"Product2","Group2"` is 67 (Calc!B16); a field not in the pivot is `#REF!` (MG `"Product","A"`, MB
  `"Product2","Group2"`), and so is a group label on MS, whose Product2 has custom subtotals sum and count only.
- Date levels (probes/t4170-pivot-date-levels Calc, against DC: rows Years, Quarters, Date): `"Years","2019"` and the
  number 2019 are 286, `"2020"` 96; pairs and the triple work (`"2019"` and `"Qtr1"` 262, `"2020"` and `"Qtr1"` 32,
  `"2020"` and `"Jan"` 32, `"2019"`, `"Qtr1"`, `"Jan"` 258); `"2019"` with `"Jul"`, a month with no 2019 record, is
  `#REF!`. An inner label alone that sits under both years is `#REF!` (`"Quarters","Qtr1"`, `"Date","Jan"`), while
  each level's `<1/1/2019` alone is 1 (it sits under `<1/1/2019` only), `"Years",">12/31/2020"` 128, `"Years","2021"`
  and `"Date",DATE(2019,1,10)` `#REF!`, and the grand total 511. With Date alone on the rows (DM) `"Date","Jan"` is
  290 and `"Years","2019"`, a field off the axes, `#REF!`; with Years on the columns (DX) the pair `"2019"`, `"Qtr1"`
  is 262; on DS, whose Years has custom subtotals sum and count only, `"Years","2019"` is `#REF!`.
Evidence: probes/t4170-pivot-manual-groups Calc!B1:B19 and probes/t4170-pivot-date-levels Calc!B1:B21 (excel-graph,
2026-11-07, T-4170).
Status: VERIFIED (excel-graph) for these pivots only. That an inner item alone answers exactly when it sits under one
outer item fits both cases, but each probes one pivot, so as a general rule it is UNVERIFIED (T-4154's two-fields-per-
axis shapes, SPEC-0015 unknown 10). `#REF!` for an item of a field with custom subtotals only is seen on grouped fields
alone; on an ungrouped field it is UNVERIFIED (T-4802). Implemented in: nothing yet. The engine answers the grand
totals, DX's pair and MG's and MB's items (the manual-group pivots render since T-4171), and `#REF!` for every item on
DC, MC, MT and MS, whose rows hold two fields (SPEC-0015 unknown 10, T-4154).

## GETPIVOTDATA on real-model pivots: an omitted item is the blank item, nested axes' grand total (T-2662)
Rule: an omitted item argument picks the field's blank item, the `(blank)` row of records whose value is missing (`<m/>`):
`GETPIVOTDATA("For Review",Pivot!$C$9,"Year","2021","Remediation Status",)` is the 2021 `(blank)` cell, 244445.62. A pivot
with two row fields and two column fields (outline form, subtotals shown) answers its grand total with no pairs
(`GETPIVOTDATA("Sum of Qux",B32)` is 57). Two pivots of one cache can total the same records differently in the last
bits: the 2021 total of the pivot at Pivot!C3 (Year columns only) is 1738748.6999999997, as is
`GETPIVOTDATA("For Review",Pivot!$C$3,"Year","2021")`, while the pivot at Pivot!C9 (with Remediation Status rows) shows
1738748.7000000002 for its 2021 grand total; folding the picked cache records in cache order gives the C3 values
(SPEC-0015 unknown 11, one data point).
Evidence: calipers goldens (desktop Excel cached values): pv_goldens/pv23_041 Liability Summary C5:G7 (the pivots on sheet
Pivot, C3:G5, C9:G13), roundtrip/formula_stress_test Database!B16 (pivot GPD at B32:O45).
Status: VERIFIED (desktop cached values). `"(blank)"` picks the blank item of an ungrouped field too (probes/t4236-pivot-date-groups,
excel-graph, T-4260; above). UNVERIFIED: whether `""` or a blank cell also pick it (ours `#REF!`), and any pair on a pivot with two fields on an axis (ours `#REF!`, SPEC-0015 unknown 10, T-2085).
Implemented in: `fn_getpivotdata`, `Model::pivot_blank_item`, `nested_axis`
(crates/rustsheet-engine/src/functions/lookup_and_reference/getpivotdata.rs, pivot_index.rs, T-2662).
Tasks: T-2662, T-2085

## Pivot tables and structural edits
Rule (engine; Excel UNVERIFIED): a whole-row or whole-column insert at or before a pivot's first line, or a delete wholly
before it, shifts the pivot with its cells; an insert or delete after its last line leaves it; a delete of every one of its
lines drops it with its cells (GETPIVOTDATA against it is then `#REF!`). Any other insert or delete that meets its lines (an
insert inside it, a delete of its first line, an inner line or its last line) is refused with an error and changes nothing:
SPEC-0015 states that Excel refuses an edit that would change part of a pivot, and refusing is the engine's way of matching
that (as for an edit that would break an array formula), never a silently cut pivot. A sheet rename keeps the sheet's pivots
(they are keyed by sheet id) and rewrites the cache sources naming that sheet; a sheet deletion removes its pivots and undoing
it brings them back. A row or column move shifts a pivot that lies wholly inside the moved lines, or wholly inside the zone they jump over, with its cells; a pivot wholly outside the moved group and the zone it jumps over stays where it is; a move that would split it (some lines in the moved group or zone, some not) is refused with an error and changes nothing, as a move that splits a merged cell is (T-2401).
Evidence: none from Excel yet. The formula oracle cannot make structural edits; T-2402 records which inserts, deletes and moves
Excel refuses (Graph range insert/delete or a desktop observation), including whether deleting every line of a pivot deletes it.
Status: UNVERIFIED
Implemented in: crates/rustsheet-engine/src/pivot.rs (`Model::displace_pivots`, `move_pivots`, `take_pivots_on`, `restore_pivots`,
`rename_pivot_sources`), called from the insert/delete/move actions in actions.rs and from sheet_undo.rs and new_empty.rs (T-2086).
Tasks: T-2086, T-2401, T-2402
