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
The limits are totals counted while parsing, before anything past them is allocated (CLAUDE.md directive 5).
Part names that differ only by case or by `\` against `/` are one part, so a pivot or cache reached twice that way is skipped like any other. A
sheet that loses pivots to a cap logs one line with the count, not one per pivot.
Evidence: none for Excel; this is a resource limit (SPEC-0015 "Untrusted-input limits").
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/pivot.rs (`CacheBuilder`, `MAX_*`), crates/rustsheet-xlsx/src/import/pivots.rs

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
  accepts it). CountNums, Product, Var, VarP, StdDev and StdDevP over an empty intersection are not probed (the case builder cannot write them): ours stay `#REF!`.
- `pivot_table` may be any cell of the pivot (`$C$6`, inside the data area) or a range covering it
  (`$A$3:$E$7`); a cell outside every pivot (`$H$30`, or the source data `Data!$A$1`) is `#REF!`.
- An array of items lifts: `"Region",{"East";"West"}` spills 22, 27 and `"Product",{"A","B","C"}` spills
  34, 12, 3.
Evidence: probes/getpivotdata-real-pivot (oracle: excel-graph, 2026-10-16, T-1705). The pivots are written
by the case builder (`pivot_tables`, tools/src/rustsheet_tools/pivot.py); excel-graph rendered them cell for
cell as the builder does (the builder also writes the rendered cells into the sheet). probes/pivot-rendered-cells
(T-2084) reads every cell of both pivots through `=P!A3` and `ISBLANK`; excel-graph opens the workbook and agrees with
the layout (caption, `Row Labels`/`Column Labels`, items, aggregates, grand totals, West/C blank: `ISBLANK` true,
reference 0). UNVERIFIED: whether Excel re-rendered the pivot under `refreshOnLoad` or kept the builder's cells, since
the probe cannot tell the two apart (T-2327 adds a control with stale rendered cells).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). Not probed: several data fields (the
`Values` field), two row fields and their subtotals, filters, hidden items, dates and numbers as items, and
pivots in tabular or outline form.
Engine (SPEC-0015): the workbook model holds pivot caches and pivot tables (`pivot.rs`, T-2081), read from xlsx by
`crates/rustsheet-xlsx/src/import/pivots.rs` (T-2082). The importer skips, whole and logged, a cache with an OLAP,
external, consolidation or scenario source, a calculated field, no records part
(`saveData="0"`), a bad index, or a value it cannot read (an unknown error name, a `<d>` date before 1900, which
includes time-only dates if Excel writes them so), and a pivot whose data field is shown as a calculation
(`showDataAs`, such as % of total), whose index points past its cache, whose `axis` or `subtotal` is unknown, which has
no `location` or not one `pivotField` per cache field, or whose parts are reached outside the package (absolute,
escaping or External targets), twice, or through a duplicate `cacheId`: GETPIVOTDATA against it is `#REF!`.
UNVERIFIED: what Excel answers for each of these (presumably it computes them, refreshing a cache saved without
records); a known gap, not a deviation. A cache `<d>` date is converted to the workbook's date system (T-2393); one before
the 1904 epoch in a `date1904` workbook has no serial there and skips the cache like a pre-1900 date (UNVERIFIED: Excel's answer). The 1904 serial is the 1900 sum minus 1462, so a timestamp's fraction can differ by one ulp from the 1904 sum's; compute the date part in the target system before pivot date items are matched (T-2085). The 1904 values are derived from the 1462-day epoch offset, not from a 1904 pivot workbook.
GETPIVOTDATA (T-2083, `functions/lookup_and_reference/getpivotdata.rs`, `pivot_index.rs`) aggregates the cache records the
pairs pick, in cache order, rather than reading the rendered cell, and passes every cell of probes/getpivotdata-real-pivot.
Until T-2085 probes them it answers `#REF!` for SPEC-0015's unknowns, never a guess: several data fields, page fields or
hidden items, a pair on a pivot with two fields on one axis, a reference meeting two pivots, `""` or an omitted
data_field, a number, date or
boolean as an item (or an item that matches two cache items differing only in case), the same field twice with the same
item, an error in a field/item pair, an empty intersection of any aggregate but Sum, a picked record whose data-field value
is not a number, a number or boolean data_field, and an array data_field, pivot_table or field name (an array of items
lifts). An error in a single-cell pivot_table propagates even inside a pivot, and an overflowing aggregate is `#NUM!`
(both unprobed, SPEC-0015 unknowns 13 and 16). Count pivots of an all-numeric field match
Excel on gen-m6 8641fa604c5b8e14, b304d8cf05adcc5e and df85bc699c7e87bc (T-2674; they cannot tell COUNT from COUNTA).
Max, Min, Product, Var and StdDev pivots are unprobed (UNVERIFIED: the worksheet functions' algorithms over the picked numbers).
Structural edits: see "Pivot tables and structural edits" below. xlsx export drops pivots.

## Grouped pivot fields import as their group items (T-4225)
Observed (Excel-authored caches, docProps/app.xml Microsoft Excel 16.0300): Excel stores no record values for a grouped
field. A `<cacheField>` grouped in place carries `<fieldGroup par base>` after its `<sharedItems>`; a grouping field Excel
adds (`Quarters`, `Years`, `Hours`, `Days`) is a `<cacheField databaseField="0">` written after every database field,
with `<fieldGroup base>` naming the database field it groups; the records part holds the database fields only. Each group
lists its labels in `<groupItems>` (`<s>` items), and a pivot field's `<item x>` indexes those labels, not the base
field's items. The date layouts are fixed: a first item for values below `startDate` (`<1/1/2019`), the calendar part, and
a last item for values past `endDate` (`>1/14/2019`):
- `groupBy="minutes"` 62 items `:00` to `:59`; `hours` 26 items `12 AM` to `11 PM`; `days` 368 items `1-Jan` to `31-Dec`
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
to its i-th `<x v>`. A blank base value goes in item 0 (the `(blank)` label above). The importer skips the cache
(logged) for a day group of more than one day, a date group in a 1904 workbook, an item count other than the layout's,
a range group over text, a database field after a grouping field, and a base that is not a database field.
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
- A group with no record is not shown (`showAll="0"`) and GETPIVOTDATA on its label is `#REF!`.
- GETPIVOTDATA on the grouped field: a text item is the label (`"0.3-0.4"`), case-folded for the field name (`"n1"`),
  not trimmed (`" 0.3-0.4"` is `#REF!`) and not read as numbers (`"0.30-0.40"`, `"0.3"` are `#REF!`); a reference to a
  rendered label cell works, and an array of labels lifts. A **number** item picks the group whose lower bound equals it,
  within the same rounding (0, 0.2, 0.6, 0.6000000000000001, 0.7, 0.7000000000000001 and 0.9 by 0.1; 1.1, 1.3,
  1.3000000000000003 from 1; 3 by 3), and `endNum` itself picks the `>end` item (1 gives the `>1` total); a number
  inside a group but not its lower bound (0.35, 4), a number inside the `<start` group (-0.5), TRUE, an omitted item and
  `"(blank)"` are `#REF!`. `"Sum of S1"` works as data_field as on an ungrouped pivot.
- Quirk: a `<groupItems>` list one item longer than the layout (6 items for 0 to 9 by 3, whose layout is `<0`, `0-3`,
  `3-6`, `6-9`, `>9`) is not refused: Excel relabels the groups with its own labels (`0-3`, not the file's `0-2`) and
  lists them as `<0`, `>9`, `0-3`, `3-6`, `6-9`; GETPIVOTDATA on the file's labels is then `#REF!` except `<0` and `>9`.
  Only pivot 3's relabel shows Excel's own labels (`0-3`, `3-6`, `6-9`). UNVERIFIED (T-4236): the label format for
  decimal data (`0.3-0.4`) and that it is formatted General; the probe's decimal labels already equal what Excel would
  write, so they show nothing. T-4236 can probe them with wrong-text labels of the right count.
- excel-graph lays a grouped pivot out itself on load (the case builder writes no rendered cells for one, and the
  probe's `=P!A3&""` reads show Excel's layout), so these are Excel's own groupings, not the file's.
Evidence: probes/t4228-pivot-range-groups (excel-graph, 2026-11-04, T-4228): five pivots whose S values are distinct
powers of two, so each group's sum names its records. Status: VERIFIED (excel-graph). Engine (T-4235): `NumberGroups`
groups by the rule, picks a number item by its lower bound, and relabels a list of the wrong length with its own
labels (`General(start + k*interval)-General(start + (k+1)*interval)`, the end groups first); a group with no record
is `#REF!` on a numeric range field only (an empty item of an ungrouped field keeps its 0, unprobed). The importer and
the eval adapter share it (`group_by_numbers`); the adapter has the engine lay a grouped spec pivot out (captions, the
groups that hold a record, grand total). The 15-digit rounding of q, not of v, is a structural model: the probe's
points fit both (T-4235 Outcome).
UNVERIFIED (T-4236): the date groups (a time of 23:59:59.5 or later, serials 1 to 60), which group item a blank goes in
(item 0 by the label; one item for certain by the item lists), and GETPIVOTDATA on a date group's label, a year as a
number, or a blank.
Implemented in: crates/rustsheet-xlsx/src/import/pivots/groups.rs (`apply_groups`), import/pivots.rs (`read_definition`),
crates/rustsheet-engine/src/pivot/number_groups.rs (`NumberGroups`, `group_by_numbers`), pivot/layout.rs (`row_layout`),
functions/lookup_and_reference/pivot_index.rs (`pivot_number_item`, `shown_group`).
Tasks: T-4225, T-4228, T-4235, T-4236

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
Status: VERIFIED (desktop cached values). UNVERIFIED: whether `""`, a blank cell or `"(blank)"` also pick the blank item
(ours `#REF!`), and any pair on a pivot with two fields on an axis (ours `#REF!`, SPEC-0015 unknown 10, T-2085).
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
