# LibreOffice Calc pivot tables ("DataPilot"): source research for RustSheet

> Research spike of 2026-11-04 (see plan.md in this folder). `<scratch>/` paths were session scratch files and are not committed. Hypotheses here are not Excel evidence until an oracle probe confirms them.

**Source:** `github.com/LibreOffice/core` (GitHub mirror; git.libreoffice.org is blocked). Shallow sparse clone.
**Commit:** `b1653ced6208ca71c4e0defd572fb43cb65050fc` ("Resolve tdf#163552 - Redesign the Macro Security dialog", 2026-11-04).
All `file:line` references are at this commit. Paths are relative to the repo root.
**Licence:** MPL-2.0. This document describes behaviour and quotes short comments as evidence. **Do not copy any code into RustSheet.**
**Clone:** `<scratch>/lo`. The checkout includes `sc/source/core/data`, `sc/inc`, `sc/source/filter/{oox,excel,xcl97,inc}`, `sc/source/core/tool`, `sc/source/ui/dbgui` and `sc/qa/unit/*.cxx`. `oox/source/xls` no longer exists: the xlsx pivot import lives in `sc/source/filter/oox`.

> **How to read this.** LibreOffice is *not* an oracle. Its behaviour often differs from Excel's, and every LO choice listed below is a **hypothesis to probe in Excel**, not evidence. The Excel-authored fixtures in §5 *are* potential oracle evidence, because they contain Excel's own rendered cells and `rowItems`/`colItems`.

---

## 0. Executive summary

* **Pipeline.** Source range → `ScDPCache` (per-column sorted unique items plus per-row item ids) → `ScDPFilteredCache` (row visibility after page/hidden-member filters) → `ScDPSource` (the UNO-ish "DataPilotSource" with dimensions/hierarchies/levels/members) → result trees `ScDPResultMember`/`ScDPResultDimension` (row and column headers) crossed with `ScDPDataMember`/`ScDPDataDimension` (cells) holding `ScDPAggData` accumulators → post-passes (UpdateDataResults → AutoShow (Top N) → re-calc → SortMembers → UpdateRunningTotals = "show values as") → `getResults()` sequences → `ScDPOutput` writes the cells.
* **xlsx import recomputes the table and throws away Excel's rendered cells and `rowItems`/`colItems`** (`pivottablebuffer.cxx:1331`). That is why LO round-trips look different from Excel. RustSheet should instead treat Excel's rendered cells and `rowItems`/`colItems` as an oracle.
* **Not supported or dropped by LO.** Calculated **items** are not supported at all. Excel 2010+ show-values-as types (`% of parent row/column/parent total`, `% running total`, `rank`), stored in `x14` extLst, are not read. Captions (`dataCaption`, `grandTotalCaption`, `rowHeaderCaption`, `errorCaption`, `missingCaption`) are parsed but unused. Label/value/date filters are dropped; only Top-N count is kept. `showError`/`missingCaption` are ignored. On export, `showDataAs` and numeric range groups are not written.
* **Calculated fields** were added recently (2025–26). LO evaluates them on the **SUM** of referenced fields at every cell, including totals, as Excel does. A reference to a non-measure field becomes 0. Cycles give `#CIRC`-style errors.
* **Key Excel quirks LO encodes** (to verify):
  * Item dedup is case-insensitive.
  * Default item order: numbers < strings < errors < blank.
  * Errors in a value field poison every aggregate except COUNT (CNT2).
  * Text is ignored by SUM/AVG.
  * Top-N includes ties.
  * Errors sort last under sort-by-value.
  * Numeric-group labels use "start-end", with end = start+size-1 for integer-only data.
  * The last group absorbs the end value.
  * Date-group "days" use a leap year, so Feb-29 is always slot 60.
  * Duplicate header names get suffixes "2", "3", … (case-insensitive).

---

## 1. Architecture

### 1.1 Cache: `ScDPCache` (`sc/inc/dpcache.hxx`, `sc/source/core/data/dpcache.cxx`)

**Per-source-column `Field` contents:**
* `maItems`: unique values in sorted order.
* `maData`: for each source row, the *order index* into `maItems`.
* `mnNumFormat`: the field's number format.
* Optional `GroupItems` (for numeric or date grouping applied in place).

There are also extra *group fields* (`AppendGroupField`, `dpcache.cxx:1397`) and **calculated fields** (`CalculatedField`, `dpcache.cxx:74`: name + token array + index; `SetCalculatedField`, `dpcache.cxx:1328`).

**Cell → item conversion** (`initFromCell`, `dpcache.cxx:162`):
* An error cell becomes an `Error` item whose string is the error text, if it is a valid OOXML error; otherwise it becomes `#N/A`.
* A numeric cell becomes a `Value` item. The cell's number format is recorded.
* A non-empty cell becomes a `String` item, using the *raw* string.
* An empty cell becomes `Empty`.
* Booleans are numeric in LO (no separate type). Excel's cache has `<b>` items; verify how Excel orders and aggregates them.

**Item ordering** (`processBuckets`, `dpcache.cxx:270-320`):
1. Sort by value using `ScDPItemData::operator<` → `Compare` (`dpitemdata.cxx:30-72`).
2. Assign order indices. Neighbours that are `IsCaseInsEqual` (`dpitemdata.cxx:187`) share an index, so **"A" and "a" collapse into one item**; the first in sorted order survives. Numeric equality uses `approxEqual`.
3. Store the per-row index.
4. Unique the items.

Type order comes from the enum `GroupValue=0, RangeStart=1, Value=2, String=3, Error=4, Empty=5` (`sc/inc/dpitemdata.hxx:33`). The comment at `dpitemdata.cxx:34` reads: *"group value, value and string in this order. Ensure that the empty type comes last."* So **numbers < text < errors < blank**. Strings compare with the locale collator (`ScGlobal::GetCollator().compareString`, `dpitemdata.cxx:67`), which is case-insensitive at primary strength.

**Other cache behaviour:**
* *Trailing empty rows* (`dpcache.cxx:560-575`, `457-466`). The range is shrunk to the data area. If trailing empties exist, one `Empty` item is appended last.
* *Field number format* (`dpcache.cxx:448-450`): "Only take non-default number format". The last non-General format seen in the column wins. `GetNumberFormat` has a TODO at `dpcache.cxx:1099`: *"Find a way to determine the dominant number format in presence of multiple number formats in the same field."* **Excel stores `numFmtId` per cacheField. Probe which cell's format Excel picks.**
* *Header labels:*
  * An empty header becomes "Column X" (`createLabelString`, `dpcache.cxx:148-159`). Excel instead refuses to create the pivot ("PivotTable field name is not valid").
  * Duplicate headers are normalised case-insensitively by appending 2, 3, … (`normalizeAddLabel`, `dpcache.cxx:357-377`). The label "Data" is reserved first for the data-layout field (`dpcache.cxx:385`). Excel also suffixes duplicates ("Amount2"); **probe exact Excel naming and case rules**.
* *Locale-independent formatting for matching* (`GetLocaleIndependentFormat`, `dpcache.cxx:1216-1233`). Dates become ISO `YYYY-MM-DD`, times `HH:MM:SS`, datetimes ISO; everything else uses General, en-US. This is used for GETPIVOTDATA matching.
* *Formula cells* are interpreted before the cache is built (`dpcache.cxx:538-542`; `MacroInterpretIncrementer`). The cache can be rebuilt from inside GETPIVOTDATA.

### 1.2 Table data / source abstraction

* **`ScDPTableData`** (`dptabdat.cxx`): abstract data source. `CalcResults` iterates the filtered rows and calls `ProcessRowData` → `pColRoot`/`pRowRoot->ProcessData`. `Compare(nDim, id1, id2)` compares cache order ids (`dptabdat.cxx:279`), so member "name" order is the cache sort order.
* **`ScSheetDPData`** (`dpshttab.cxx`) is a sheet range or named range. `ScDatabaseDPData` (`dpsdbtab.cxx`) is a DB source. `ScDPGroupTableData` (`dpgroup.cxx`) is a decorator adding group dimensions; its index space is `[source 0..N-1][groups N..N+G-1][calc fields][DataLayout]` (`dpgroup.cxx:471-513`).
* **`ScDPFilteredCache`** (`dpfilteredcache.cxx`) handles row visibility.
  * `fillTable(query, bIgnoreEmptyRows, bRepeatIfEmpty)` is at line 129. `RepeatIfEmpty` fills blank category cells with the previous non-empty value; this is an LO option, not Excel's.
  * `filterByPageDimension` is at line 273.
* **`ScDPSource`** (`dptabsrc.cxx`) implements the `com.sun.star.sheet.DataPilotSource` model: Dimensions → Hierarchies → Levels → Members.
  * `ScDPLevel` holds sort info (`EvaluateSortOrder`, `dptabsrc.cxx:2034`), AutoShow info, layout info (compact/outline/tabular, add-empty-lines) and subtotal functions.
  * Its default sort is `NAME` ascending (`dptabsrc.cxx:2018`).
* **`ScDPSaveData` / `ScDPSaveDimension` / `ScDPSaveMember`** (`dpsave.cxx`) is the persisted user config (the analogue of `pivotTableDefinition`). `WriteToSource` pushes it into `ScDPSource`.
  * In MANUAL sort mode, members listed in the save data get positions 0, 1, 2… (`dpsave.cxx:598-615`).
  * `ScDPMember::Compare` (`dptabsrc.cxx:2607-2630`): members with positions come first, in position order; *"members with specified positions come before those without"*; the rest follow by cache order. The xlsx importer creates save members in `<items>` order (`pivottablebuffer.cxx:867-897`), so **Excel's `<items>` order becomes the manual order**. That matches Excel, where `sortType="manual"` means "use `<items>` order".
* **`ScDPObject`** (`dpobject.cxx`) is the pivot table object: owns save data, output and source. It also provides `GetPivotData` (1543) and `ParseFilters` (1905, the legacy one-string GETPIVOTDATA form).

### 1.3 Result computation (`dptabres.cxx`, `dptabsrc.cxx:916-1063`)

**`ScDPSource::CreateRes_Impl` (`dptabsrc.cxx:916`) runs these steps in order:**
1. **Data layout placement.** If there is more than one data field and the data layout field has no orientation, it is forced to ROW (`dptabsrc.cxx:922-928`).
   * In Excel the default for multiple data fields is "Σ Values" on **columns** (`dataOnRows=0`), so this default differs. The xlsx import sets the orientation explicitly via field index `-2` (`OOX_PT_DATALAYOUTFIELD`, `pivottablebuffer.cxx:78`).
2. **Measures.** `AddDataDimsToResultData` (`dptabsrc.cxx:729`) builds the measure list (function, `DataPilotFieldReference`, ref orientation, names).
   * Calculated fields add hidden SUM measures for their dependencies (§1.6).
   * If any measure uses a reference type that needs other members (difference / % of / running total), LateInit is disabled, so all members are fully initialised (`dptabsrc.cxx:875-884`).
3. **Page selections.** These seed `ScDPInitState`. Page-field selections restrict related members (`dptabsrc.cxx:936-947`).
4. **Grand totals.** They are shown only if the option is on **and** at least one real (non-data-layout) field exists on that axis (`dptabsrc.cxx:949-958`): *"Show grand total columns only when the option is set *and* there is at least one column field."* This matches Excel, where there is no grand total row when there are no row fields.
5. **Build trees.** Column and row result roots (`ScDPResultMemberFull`) are built with `InitFrom` (members in global sort order, `dptabres.cxx:3118-3214`). The innermost row level has layout flags disabled (`dptabsrc.cxx:969-973`).
6. **Size guard.** A pre-check counts the minimum number of output rows and columns, giving `mbResultOverflow` if the result is too big (`dptabsrc.cxx:993-1003`).
7. **Filter and accumulate.** `FilterCacheByPageDimensions()` runs, then `mpData->CalcResults()` feeds every visible source row into the trees (`ProcessData`). Each `ScDPDataMember` cell has per-measure `ScDPAggData`, plus a child chain for multiple subtotal functions.
8. `CheckShowEmpty`.
9. `UpdateDataResults` calls `ScDPAggData::Calculate` for every cell.
10. **AutoShow (Top N), if used.** `DoAutoShow` marks members auto-hidden, then **all results are reset and recomputed** from data for the remaining members only (`dptabsrc.cxx:1023-1036`). So totals reflect the Top-N subset, like Excel's default (no "include filtered items in totals").
11. `SortMembers` sorts by data, using the *original* results (not running totals).
12. `UpdateRunningTotals` computes running totals, differences, percentages and INDEX. The comment at `dptabsrc.cxx:1045-1051` explains that these use **sorted** values: *"Running totals and relative differences need to be done using the sorted values. Column/row percentages and index values must be done after sorting"*.

**Aggregation: `ScDPAggData::Update` (`dptabres.cxx:391-464`)**
* Empty values are skipped.
* For every function **except COUNT (CNT2)**:
  * An **error value poisons the cell**: `nCount=-1` (`dptabres.cxx:410-413`, *"-1 for error (not for CNT2)"*).
  * **Strings are ignored** (`dptabres.cxx:414-415`).
* CNT2 *"counts everything, incl. strings and errors"* (`dptabres.cxx:408`).
* Overflow in SUM or PRODUCT also produces an error (`SubTotal::SafePlus`, `SafeMult`).
* STDEV/VAR use a Welford accumulator. MEDIAN keeps a sorted vector (an LO-only function, not in Excel pivots).

**`Calculate` (`dptabres.cxx:466-610`)**
* Error conditions by function:
  * SUM/PROD/COUNT/COUNTA: only real errors.
  * AVE/MED/MAX/MIN/STDP/VARP: *"no data is an error"*.
  * STD/VAR: *"need at least 2 values"*.
* Results:
  * An empty cell (count 0) is EMPTY, not an error. *"Now, for compatibility, empty "average" results are counted as 0."* (`dptabres.cxx:590-596`).
  * EMPTY results are output as blank cells.
  * Errors are output as `#VALUE!` regardless of their origin (§1.4).
* **Excel to verify:**
  * Excel shows the *first/most significant actual error* (for example `#DIV/0!`), or `errorCaption` when `showError=1`.
  * Excel's StdDev with 1 value gives `#DIV/0!`.
  * A data field with only text under Sum gives 0 or blank?
* Mapping from data-field function to internal function (`ScDPUtil::toSubTotalFunc`, `dputil.cxx:391-414`): `COUNT` (Excel `count`, "Count") becomes CNT2 (count all non-empty); `COUNTNUMS` becomes CNT.

**Errors and sorting**
* Errors always sort at the end, in both directions (`lcl_IsLess`, `dptabres.cxx:187-210`).
* Members with no data sort as 0 (`dptabres.cxx:204`).
* Hidden members get the largest order (`dptabres.cxx:249-251`).

**Subtotals**
* The number of subtotals comes from `GetSubTotalCount` (`dptabres.cxx:1359-1387`). If there are manual functions, a hidden "automatic" subtotal is prepended *"used for calculation, but not for display, needed for sorting"* (`dptabres.cxx:1376-1381`). The grand total *"is always automatic"* (`dptabres.cxx:1367`).
* Multiple subtotal functions use the force-function mechanism (`ScDPSubTotalState.eColForce` / `eRowForce`). When the row-forced and column-forced functions differ, the cell is skipped (`dptabres.cxx:400-402`), so a cell at the cross of a row "Avg" subtotal and a column "Sum" subtotal is empty. **Probe Excel.**
* **Subtotal in title row** (`IsSubTotalInTitle`, `dptabres.cxx:1267-1287`): in outline layout with subtotals at top, the subtotal is shown in the member's header row **only if exactly one subtotal row would be inserted**. That means a single subtotal function × (one measure, or the data layout not below this level). Otherwise a separate subtotal row is emitted.
* Sizes: `GetSize` (`dptabres.cxx:1289-1324`) adds one row for the outline title (unless the subtotal is in the title), one row per visible subtotal × measure, and one row for "insert blank line" (`IsAddEmpty`).

### 1.4 Output rendering: `ScDPOutput` (`dpoutput.cxx`) + `ScDPOutputGeometry` (`dpoutputgeometry.cxx`)

**What gets written:**
* Member header sequences (`getResults()` per level) and the 2-D data result matrix are written straight to cells.
* A row header cell gets the numeric value when `MemberResultFlags::NUMERIC` is set, otherwise the text caption (`HeaderCell`, `dpoutput.cxx:517-548`).
* Numeric group titles are forced to text: *"Numeric group dimensions use numeric entries for proper sorting, but the group titles must be output as text."* (`dptabres.cxx:1507-1512`).
* A member with a custom layout name is always text (`dptabres.cxx:1521`).

**Data cells**
* `ScDPOutput::DataCell` (`dpoutput.cxx:468-515`): **every error is written as `FormulaError::NoValue` (#VALUE!)** (`dpoutput.cxx:473`).
* A third-party report (§4) shows Excel rendering `#DIV/0!` where LO shows `#VALUE!` for a calculated field.

**Layouts**
* **Compact.** Supported per row field via `DataPilotFieldLayoutMode::COMPACT_LAYOUT` (`dpoutput.cxx:370-383`, `GetColumnsForRowFields` 612-626, `outputRowHeader` 811-890).
  * Consecutive compact row fields share one column with an indent of 13 px per level (`dpoutput.cxx:841-850`).
  * A single "Row Labels" / "Column Labels" multi-field header cell (`MultiFieldCell`, `dpoutput.cxx:550-568`) is used when any row field is compact and there is more than one row or column field.
  * The importer marks a field compact only when `outline && subtotalTop && compact` (`pivottablebuffer.cxx:296`).
* **Outline.** Title row plus subtotals at top or bottom (`OUTLINE_SUBTOTALS_TOP` / `_BOTTOM`).
* **Tabular.** No title row; the child starts on the same row (`dptabres.cxx:1561-1574`).
* Page fields are rendered **above** the table: one row per page field, then one empty row (`CalcSizes`, `dpoutput.cxx:650-655`), plus an optional filter-button row.
* Page value text:
  * "- all -" (`SCSTR_ALL`).
  * `(empty)` for a blank item.
  * "- multiple -" (`dpoutput.cxx:736-748`).
* **Excel instead shows "(All)", "(blank)" and "(Multiple Items)".**
* The **extra header row** is inserted only when there is no column field: *"Insert an extra header row only when there is no column field."* (`dpoutput.cxx:644`).
* The data description ("Sum - X", or "Data") goes in the top-left corner. When there is exactly one data field and no column fields, it is also written above the data column (`dpoutput.cxx:1003-1013`).

**Caption strings** (`sc/inc/globstr.hrc`)

| Meaning | LO string | Excel string (to verify) |
|---|---|---|
| Grand total (`STR_TABLE_ERGEBNIS`) | "Result", shown as "Total Result" (`globstr.hrc:149`) | "Grand Total" |
| Subtotal | "\<item\> Result", i.e. caption + " " + measure string (`dptabres.cxx:1606-1609`) | "\<item\> Total" |
| Data-layout field (`STR_PIVOT_DATA`) | "Data" (`globstr.hrc:164`) | "Values" |
| Measure name | "Sum - Field" (`ScDPUtil::getDisplayedMeasureName`, `dputil.cxx:380-389`) | "Sum of Field" |
| Blank item (`STR_EMPTYDATA`) | "(empty)" (`globstr.hrc:227`) | "(blank)" |
| Compact header | "Row Labels" / "Column Labels" (`globstr.hrc:167-168`) | same (Excel's) |

**Geometry**
* In `ScDPOutputGeometry`, **only one data field means the data layout field is not counted** (`adjustFieldsForDataLayout`, `dpoutputgeometry.cxx:200-218`).
* In compact mode all row fields sit in one column, and column-field buttons start one column right of the table's first column (`dpoutputgeometry.cxx:99-100`, `125-126`).

**Spill**
* LO now has spill-like collision detection when the table grows over existing cells (`Output(rOldRange, bCheckForSpill)`, `dpoutput.cxx:1017+`; tests `testPivotTableSpill*`).
* Excel instead shows the "A PivotTable report cannot overlap…" error and does not refresh.

### 1.5 Grouping (`dpgroup.cxx`, `dputil.cxx`, `dpdimsave.cxx`, `dpnumgroupinfo.cxx`)

* **Numeric range groups** (`ScDPNumGroupInfo`: start, end, step, autoStart, autoEnd, integerOnly, dateValues).
  * **Group start value** (`ScDPUtil::getNumGroupStartValue`, `dputil.cxx:151-184`):
    * Below start → `-inf`, label `"<start"`.
    * Above end → `+inf`, label `">end"`.
    * Otherwise `start + floor((v-start)/step)*step`, using `approxFloor`.
    * Key quirk (`dputil.cxx:166-180`): *"A group that would consist only of the end value is not created, instead the value is included in the last group before."* For dates, *"the end value is instead treated as above the limit if it would be a group of its own."*
  * **Label** (`lcl_GetNumGroupName`, `dputil.cxx:209-252`):
    * Numbers: `"start-end"` with no spaces. Dates: `"start - end"` with spaces.
    * *"The second number of the group label is (first number + size - 1) if there are only integer numbers, (first number + size) if any non-integer numbers are involved. Exception: The last group (containing the end value) is always shown as including the end value (but not for dates)."*
    * The label is clipped to the end value when autoEnd is off.
    * Numbers are formatted with `rtl_math_StringFormat_Automatic` (shortest round-trip).
  * **Excel to verify:** Excel's labels are "1-10", "11-20", "<1" and ">100", and Excel stores `groupItems` explicitly in the cache. **Read them from the file instead of regenerating.**
* **Date groups** (`DataPilotFieldGroupBy`: SECONDS, MINUTES, HOURS, DAYS, MONTHS, QUARTERS, YEARS).
  * **Date-part values** (`ScDPUtil::getDatePartValue`, `dputil.cxx:283-357`):
    * Start and end are inclusive. *"End date without a time value is included, with a time value it's not"* (`dputil.cxx:287-288`, also `dpgroup.cxx:150-151`).
    * Outside the range gives `DateFirst` (-1) or `DateLast` (10000), labelled `"<date"` / `">date"`.
    * DAYS counts days 1..366 in a fixed leap-year frame: a non-leap date at or after Mar 1 is shifted by +1, so Feb-29 is always slot 60 (`dputil.cxx:340-349`; `SC_DP_LEAPYEAR = 1648`, `dputil.cxx:33`).
    * Time parts use `tools::Time::GetClock` ("do as in the cell functions, ScInterpreter::ScGetHour()").
  * **Labels** (`getDateGroupName`, `dputil.cxx:102-149`):
    * Years: the number.
    * Quarters: locale quarter abbreviation (en-US "Q1").
    * Months: locale short month name ("Jan").
    * Days: `NF_DATE_SYS_DDMMM` ("01-Jan").
    * Hours: two digits ("09"). Minutes and seconds: ":05".
    * Excel uses "Qtr1", "Jan", "1-Jan", "9 AM", ":05" (to verify).
  * Nested date groups (Years > Quarters > Months) use separate group dimensions. `IsInGroup` / `HasCommonElement` (`dpgroup.cxx:958-1056`) decide containment so empty combinations are not shown, e.g. *"a month is only contained in its quarter"* (`dpgroup.cxx:276`).
  * "Days" with step ≥ 2 is a numeric range on dates (`pivotcachebuffer.cxx:655`, `bDayRanges`).
  * xls import: *"return nothing for step count 1 - this is also a standard date group in Excel"* (`sc/source/filter/excel/xipivot.cxx:600`).
* **Manual (named) groups**: `ScDPSaveGroupDimension` / `ScDPSaveGroupItem` (`dpdimsave.cxx`).
  * Default names are "Group1", "Group2"… Ungrouped items form their own singleton group.
  * Export writes named groups first, then ungrouped source items, each as its own group entry (`xepivotxml.cxx:622-660`).
  * Excel's group field is named "Field2" (base name + 2) with items "Group1"… (to verify).
  * Import comment (`pivotcachebuffer.cxx:723-729`): *"Calc expects the names of the members of the field whose members are grouped (which may be the names of groups too). Excel provides the names of the base field items instead (no group names involved)."*
* **Shared groups.** Excel stores grouping in the *cache*, so all pivot tables on a cache share it. LO stores it per table and re-applies "reference groups" on import (`pivottablebuffer.cxx:1447-1466`). `xepivot.cxx:1671-1675` (BIFF export): *"In Excel, the pivot cache contains additional fields (i.e. grouping info, calculated fields)… do not share the cache with multiple pivot tables."*

### 1.6 Calculated fields / calculated items

* **Calculated fields** are now supported (2025–26 work).
  * `cacheField@formula` + `databaseField="0"` is imported (`pivotcachebuffer.cxx:426`; `PivotTableField::resolveCalculatedField` / `compileCalculatedFieldFormula`, `pivottablebuffer.cxx:390-450`).
  * The formula is compiled with the OOXML grammar and pivot field names as `ocDPFieldName` tokens (`ScCompiler::SetAvailablePivotFields`).
  * xlsb `PivotParsedFormula` is supported (`pivotcachebuffer.cxx:503-515`, `1039-1065`).
  * A dialog exists (`sc/source/ui/dbgui/PivotCalcFieldDialog.cxx`). Calculated fields can only go in the data area (`PivotLayoutDialog.cxx:627`, tdf#174093).
* **Evaluation semantics** (to verify each against Excel):
  * Calculated fields *"always operate on SUM of referenced fields (matching Excel behavior)"*. A referenced field visible with COUNT gets a hidden SUM copy (`dptabsrc.cxx:736-807`, `852-857`). `FindMeasureIndex` searches from the end so the hidden SUM measure wins (`dptabres.cxx:2839-2852`).
  * The formula is evaluated **per cell on aggregated sums, including subtotal and grand total cells**. It is not a sum of the row-level results (`ReplaceTokenMeasuresWithValues`, `dptabres.cxx:2854-2937`; `ScDPCache::GetCalculatedValueByToken`, `dpcache.cxx:1303`).
  * A NaN result becomes an error (`SetCalculatedResult`, `dptabres.cxx:645-658`).
  * A referenced measure in error becomes a `#VALUE!` token (`dptabres.cxx:2917-2919`).
  * A field name that is valid but not a measure (a group, row or column field) becomes 0: *"Replace with 0 to match Excel (SUM of text/group values = 0)."* (`dptabres.cxx:2926-2934`).
  * Cycles become `FormulaError::CircularReference` (`dptabres.cxx:2867-2875`).
  * After source changes, references to renamed or removed columns become `#NAME?` *"matching Excel"* (`dpdimsave.cxx:849`; test `testCalcFieldNameErrorXLSX`, `pivottable_filters_test.cxx:3071`: formula `IF(Werte2 >0,Werte2,0)` becomes `IF(#NAME? >0,#NAME?,0)`).
  * The calculated field is evaluated with the extended (hidden-measure) set, then the measures are reset so only visible ones are output (`dptabsrc.cxx:1055-1062`).
* **Calculated items** (`pivotField/items/item` with `<calculatedItems>`): **not supported anywhere.** There are no hits for `calculatedItem` in `sc/source` or `sc/inc`, and the importer skips non-`data` item types (`pivottablebuffer.cxx:871-872`). So the LO code says nothing about Excel's calculated-item semantics (solve order, interaction with subtotals). RustSheet needs its own probes.

### 1.7 Show values as (`sheet::DataPilotFieldReference`)

**Supported types.** `NONE`, `ITEM_DIFFERENCE`, `ITEM_PERCENTAGE`, `ITEM_PERCENTAGE_DIFFERENCE`, `RUNNING_TOTAL`, `ROW_PERCENTAGE`, `COLUMN_PERCENTAGE`, `TOTAL_PERCENTAGE` and `INDEX`. These map exactly to the ECMA `ST_ShowDataAs` values `normal, difference, percent, percentDiff, runTotal, percentOfRow, percentOfCol, percentOfTotal, index` (`pivottablebuffer.cxx:256`, `715-750`).

**Missing types.** The Excel 2010+ types stored in `x14:dataField@pivotShowAs` in extLst (`percentOfParentRow`, `percentOfParentCol`, `percentOfParent`, `percentOfRunningTotal`, `rankAscending`, `rankDescending`) are missing. The importer does not read extLst at all; there are no `x14` hits in `pivottable*`.

**Base item encoding.** `baseItem` `0x001000FC` means previous and `0x001000FD` means next (`pivottablebuffer.cxx:80-81`). Other values are a *cache item index* resolved to the item name (`pivottablebuffer.cxx:735-748`).

**Algorithm: `ScDPDataMember::UpdateRunningTotals`, `dptabres.cxx:2465-2837`**

*Locating the base field.* Walk the row or column result tree along the current (sorted) path until the dimension named `ReferenceField` is found.
* If the base field is not a row or column field, the cell shows an error.
* If the current cell is *above* that level (a subtotal of an outer field), there is no select-dim and the result is empty or error (`dptabres.cxx:2540-2556`).

*Running total.*
* Kept as an auxiliary value in the *first* member's agg data along the base dimension. Members are visited in final sorted order, so cells accumulate.
* If the base dimension has hidden details (a collapsed member), the result is an error, *"don't show or sum up the value"* (`dptabres.cxx:2575-2592`).
* Cells with no data show as non-empty: `SetEmpty(false) // always display`.
* Running total subtotal and grand total cells: the grand total row has no base dimension above it, so it shows **empty** (`dptabres.cxx:2670-2675`). **Excel to verify:** Excel shows blank for the running-total grand total.

*Difference / % of / % difference.*
* The base item is found by name or by relative position: previous or next of `rColVisible`/`rRowVisible`, *"without sort order applied"* (`dptabres.cxx:2681-2700`). **So previous and next use the unsorted (natural) order, not the displayed sort.** Probe Excel.
* The reference item itself shows **empty** for difference and %difference, but 100% for %-of (`dptabres.cxx:2704-2709`).
* Division by zero gives an error.
* With previous/next and no neighbour, the result is empty (`dptabres.cxx:2741-2744`).
* With a named item that is missing, the result is an error.
* Comment: *"Difference/Percentage from previous/next: If details are hidden for this member in the innermost column/row dimension... show an error value."* (`dptabres.cxx:2595-2607`).

*Row % / column % / total % / index.*
* Totals are captured as auxiliary values when the grand total or row total is visited, which always happens before use (`dptabres.cxx:2752-2771`). A total that is in error counts as 0.
* `INDEX = (value × grand total) / (row total × column total)`. A zero denominator gives an error (`dptabres.cxx:2795-2807`).
* Percentages are stored as fractions; the number format supplies the %.

**Rank / % of parent: no LO implementation exists.** Excel semantics must be probed.

### 1.8 Sorting and AutoShow (Top N)

* **Sort by name** (`ScDPGlobalMembersOrder`, `dptabsrc.cxx:1983-2011`):
  * Uses `ScDPMember::Compare`: position first (manual), then cache order.
  * MANUAL is always ascending.
  * Default level sort is NAME ascending (`dptabsrc.cxx:2018`).
* **Sort by data** (`ScDPResultDimension::SortMembers`, `dptabres.cxx:3477-3504`):
  * Uses the chosen measure's value at the member's **row total** (`GetDataRoot`).
  * Errors go last; no data counts as 0.
  * Uses `std::sort`, which is **not stable**. Ties may come out in arbitrary order; Excel probably keeps item order on ties. Probe this.
  * Column dimensions sort through `ScDPDataDimension::SortMembers` (`dptabres.cxx:4079`).
* xlsx import maps `sortType` ascending/descending + `autoSortScope/pivotArea/references/reference field=-2 (data) + x v=dataFieldIdx` to DATA sort. Otherwise it uses NAME sort (`pivottablebuffer.cxx:844-865`).
* **Top N** (`ScDPResultDimension::DoAutoShow`, `dptabres.cxx:3506-3564`; column version `4115-4182`):
  * Sorts by the measure (descending for top), keeps `nAutoCount`, and **also includes subsequent members whose value `approxEqual`s the last included one** (*"include more members if values are equal"*, `dptabres.cxx:3551`). So ties are included, and Top 3 can show 4 items. Probe Excel.
  * Applied at every level independently (`dptabres.cxx:3510-3520`).
  * Imported from `pivotField@autoShow/itemPageCount/rankBy/topAutoShow` and from `<filter type="count"><autoFilter><top10 val top>` (`pivottablebuffer.cxx:832-842`, `1004-1020`).
  * *"only simple top10 filter supported"* (`pivottablebuffer.cxx:1006`). Percent and sum Top-10 variants, label filters, value filters and date filters are all dropped on import.

### 1.9 Filtering (page fields, hidden members)

* Hidden members (`item h="1"`) become `ScDPSaveMember::SetIsVisible(false)`. Show-details (`sd="0"`) becomes `SetShowDetails(false)` (`pivottablebuffer.cxx:888-890`).
* Filtering is applied by row criteria on the cache (`FilterCacheByPageDimensions`, `dptabsrc.cxx:680-727`). Hidden row or column members also restrict data, so **totals exclude hidden items**. This is like Excel's default; `subtotalHiddenItems` is parsed but unused.
* Page fields:
  * Single selection: `pageField@item` is the *pivotField item index*, not the cache index: *"specification is wrong, XML_item is not the cache item, but the field item"* (`pivottablebuffer.cxx:1152`).
  * Multi-select: `multipleItemSelectionAllowed=1`. If exactly one item is visible it is treated as a single selection (`pivottablebuffer.cxx:630-650`).
* Page fields restrict which members appear in row and column fields only via `ScDPInitState` and group compare (`dptabsrc.cxx:936-947`).
* `showAll` maps to `ShowEmpty`, meaning "show items with no data" (`pivottablebuffer.cxx:829`).

### 1.10 GETPIVOTDATA (`interpr2.cxx:3543+`, `dpobject.cxx:1543-1612`, `dpresfilter.cxx`)

* `ScInterpreter::ScGetPivotData` (`sc/source/core/tool/interpr2.cxx:3543`):
  * The parameter count must be even and ≥ 2, otherwise `#REF!`.
  * With 2 parameters and a reference first, the old syntax applies (a single filter string parsed by `ScDPObject::ParseFilters`, `dpobject.cxx:1905`, with `'quoted'` and `[bracket;Func]` forms).
  * Numeric filter values are formatted twice: once with the cell's format (display name) and once locale-independent (ISO date / General) for matching (`interpr2.cxx:3613-3644`).
  * Comment at `interpr2.cxx:3684`: *"NOTE : MS Excel docs claim to use the 'most recent' which is not exactly the same as what we do in ScDocument::GetDPAtBlock"*. This concerns which pivot table is used when the reference touches several.
  * Errors are `#REF!` for no pivot or no match (NaN → `#REF!`).
* `ScDPObject::GetPivotData` (`dpobject.cxx:1543`):
  * The data field is matched case-insensitively (uppercase).
  * Filters are sorted *"in order of appearance, and row dimensions must come before column dimensions"* (`dpobject.cxx:1567-1570`).
  * Then `getFilteredResults` is called.
* **`ScDPResultTree` (`dpresfilter.hxx:41-56`, `dpresfilter.cxx`)**:
  * It is a tree of dimension → member (uppercased display name, plus a separate map for locale-independent values) → leaf values per measure.
  * *"The root node has two child nodes if the pivot table consists of both column and row dimensions. The first child stores the result tree that is first filtered by row dimensions then by column dimensions. The second child stores the result tree that is filtered by column dimensions only (for column grand totals)."*
  * If the same leaf name pair occurs twice, its value becomes NaN, i.e. ambiguous gives `#REF!` (`dpresfilter.cxx:166-170`).
  * Lookup *"Descend[s] into dimension member children while there is no result and exactly one dimension field with exactly one member item"* (`dpresfilter.cxx:226-228`). That gives implicit access to a leaf when an outer field has a single item.
  * A subtotal is addressed by omitting inner fields.
* **Excel to verify:**
  * Excel's GETPIVOTDATA matches items by *value*: numbers and dates as numbers, text case-insensitively.
  * It returns `#REF!` for items not visible.
  * Behaviour with hidden subtotals and calculated items is unknown.

---

## 2. Algorithms worth knowing (condensed)

| Topic | LO behaviour | Where |
|---|---|---|
| Item identity | Case-insensitive string dedup; `approxEqual` for numbers | `dpcache.cxx:279-285`, `dpitemdata.cxx:187` |
| Default item order | Numbers asc < strings (collator) < errors < blank | `dpitemdata.cxx:30-72`, `dpitemdata.hxx:33` |
| Manual order | `<items>` order first (positions), then cache order | `dpsave.cxx:596-615`, `dptabsrc.cxx:2607-2630` |
| Text in Sum/Avg/Max… | Ignored | `dptabres.cxx:414` |
| Error in data | Poisons aggregate except COUNT(all) | `dptabres.cxx:408-413` |
| Empty cell | Output blank, even for Average | `dptabres.cxx:590-604` |
| Error output | Always `#VALUE!` (LO limitation) | `dpoutput.cxx:473` |
| Sort by value | Errors last, empty = 0, unstable sort, row-total value | `dptabres.cxx:187-262`, `3477-3504` |
| Top N | Ties beyond N included (approxEqual); totals recomputed for visible only | `dptabres.cxx:3506-3564`, `dptabsrc.cxx:1023-1036` |
| Running total | Accumulates in visit (sorted) order within the base field; restarts under each outer item; collapsed members give error; grand total empty | `dptabres.cxx:2575-2675` |
| Difference prev/next | Uses unsorted (natural) order; reference item empty; none gives empty | `dptabres.cxx:2681-2744` |
| % of row/col/total, Index | Captured totals; /0 gives error; Index = v·GT/(RT·CT) | `dptabres.cxx:2747-2810` |
| Subtotal in title | Only if exactly 1 subtotal row would be produced | `dptabres.cxx:1267-1287` |
| Multiple subtotal funcs | Hidden "auto" subtotal first; row×col forced funcs that differ give empty | `dptabres.cxx:1359-1387`, `400-402` |
| Grand totals | Only if the axis has ≥ 1 real field | `dptabsrc.cxx:949-958` |
| Num group label | "a-b"; integer data gives b = a+step-1; last group absorbs end | `dputil.cxx:151-252` |
| Date day-of-year | Leap-year frame (Feb-29 = 60) | `dputil.cxx:340-349` |
| Duplicate headers | Suffix 2, 3… case-insensitive | `dpcache.cxx:357-377` |
| GETPIVOTDATA | Uppercase name match; display or ISO value; ambiguous gives `#REF!` | `dpresfilter.cxx`, `interpr2.cxx:3543` |

**How headers are produced.**
* `ScDPResultMember::FillMemberResults` (`dptabres.cxx:1457-1681`) writes, for each level, one `MemberResult` per output row or column:
  * `HASMEMBER` on the first row of a member.
  * `CONTINUE` on the rows it spans (unless "repeat item labels" is on, `dptabres.cxx:1553-1566`).
  * `SUBTOTAL` / `GRANDTOTAL` on total rows, with caption `"<item> " + measure string` (`dptabres.cxx:1606-1609`). A custom subtotal name (`pParentDim->GetSubtotalName`) replaces it.
  * The outline title occupies one row, and the children are written one row lower (`dptabres.cxx:1561-1574`).
* `ScDPOutput` then places each level in its own column. Compact levels share a column with indentation (`dpoutput.cxx:811-890`).

---

## 3. xlsx import / export

### 3.1 Import (`sc/source/filter/oox/pivotcachebuffer.cxx`, `pivotcachefragment.cxx`, `pivottablebuffer.cxx`, `pivottablefragment.cxx`, `PivotTableFormat*.cxx`)

**pivotCacheDefinition: read**
* Source:
  * worksheetSource `ref`/`sheet`/`name`.
  * Table source, which *"exclude[s] the totals row(s)"* (`pivotcachebuffer.cxx:1196`).
  * External source.
* `cacheField`:
  * `name`, `caption`, `propertyName`, **`formula`**, `numFmtId`, `sqlType`, `hierarchy`, `level`, `mappingCount`, `databaseField`, `serverField`, `uniqueList`, `memberPropertyField` (`pivotcachebuffer.cxx:423-435`).
  * `sharedItems` flags (`441-449`) and items `m s n d b e x` (`pivotcachebuffer.cxx:136-224`).
* `fieldGroup`: `par`, `base`; `rangePr` (`startDate endDate startNum endNum groupInterval groupBy autoStart autoEnd`, `pivotcachebuffer.cxx:457-475`); `discretePr`; `groupItems`.
* Cache records (`pivotCacheRecords`) are **used only to synthesise a dummy source sheet** when the source sheet is gone or external (`pivotcachebuffer.cxx:1210-1245`, `writeSourceDataCell` 822-849). Otherwise the live sheet range is used and the cache is rebuilt from the sheet.
  * **Consequence:** an out-of-date cache in the file (sheet edited without refresh) is silently refreshed by LO. Excel shows the stale cache until refresh. See tests `testPivotTableXLSX_OutOfSyncPivotTableCachedDefinitionImport*`.
* Binary dates (xlsb) comment (`pivotcachebuffer.cxx:113-119`): *"Dates before 1900-Mar-01 are stored including the non-existing leap day 1900-02-29. tools::Time values (without date) are stored as times of day…"*
* `PivotCacheItem::getName` has `// !TODO` for `n`, `d`, `b`, `e` (`pivotcachebuffer.cxx:233-237`): number names are formatted with `OUString::number`, while dates and errors give an empty name. `getFormattedName` (243-264) uses the field's number format.

**pivotCacheDefinition: ignored**
* `refreshOnLoad` (read into the model but not acted on in oox).
* `missingItemsLimit`, `invalid`, `saveData`, `tupleCache`.
* `calculatedItems` and `calculatedMembers`.
* All extLst content.

**pivotTableDefinition: read and used**
* `name`, `cacheId`, `tag`, `rowGrandTotals`, `colGrandTotals`, `enableDrill`, `showDrill` (`pivottablebuffer.cxx:1359-1366`).
* `location@firstHeaderRow`: `0` hides the header (`pivottablebuffer.cxx:1367`).
* `pivotTableStyleInfo` (`pivottablefragment.cxx:186-198`).
* `<formats>` (pivot area formats; `PivotTableFormat.cxx`).

**pivotTableDefinition: parsed into the model but unused**
* All of these (`pivottablebuffer.cxx:1074-1122`; grep shows no later use):
  * `dataCaption`, `grandTotalCaption`, `rowHeaderCaption`, `colHeaderCaption`, `errorCaption`, `missingCaption`, `showError`, `showMissing`.
  * `dataPosition`, `pageWrap`, `pageOverThenDown`, `indent`, `mergeItem`, `subtotalHiddenItems`, `visualTotals`, `showCalcMbrs`, `showEmptyRow/Col`, `showHeaders`.
  * `fieldListSortAscending`, `customListSort`, `itemPrintTitles`, `useAutoFormatting`, `apply*Formats`.
  * `dataOnRows`: the data field position is taken from `rowFields`/`colFields` containing `-2`.
* `rowItems` / `colItems` are not read at all; Excel's rendered layout is discarded.
* The destination area is cleared: *"clear destination area of the original pivot table"* (`pivottablebuffer.cxx:1331-1340`). The table is then regenerated.
* Page fields are re-laid out: *"If page fields exist, include them into the destination area (they are excluded in Excel). Add an extra blank row."* (`pivottablebuffer.cxx:1422-1425`). Excel's `location@ref` **excludes** the page-field area, which sits above it with one blank row between. `xepivotxml.cxx:1257`: *"NB: Excel's range does not include page field area (if any)."*

**pivotField: read**
* `axis` and `dataField`:
  * *"The documentation mentions a value 'axisValues' for the attribute 'pivotField@axis'. But this value is not used to mark a data field… the boolean attribute 'pivotField@dataField' is really used"* (`pivottablebuffer.cxx:270-275`).
  * The BIFF12 axis is a weird bit field (`220-227`).
* `numFmtId`, `itemPageCount`, `rankBy`, `sortType`.
* Subtotal flags:
  * Field subtotal names are inverted relative to dataField names: *"For subtotals, 'countA' means 'count all', and 'count' means 'count numbers'. On the other hand, for the data field aggregation function, 'count' means 'count all', and 'countNum' means 'count numbers'"* (`pivottablebuffer.cxx:788-794`; also 689-693).
  * *"Order of subtotals is fixed in Excel."*
  * *"Ignore the 'defaultSubtotal' flag, if explicit functions are set. This is different behaviour between XML (where 'defaultSubtotal' is set regardless of other functions) and binary formats"* (`pivottablebuffer.cxx:782-786`).
* `showAll`, `outline`, `subtotalTop`, `compact`, `insertBlankRow`, `autoShow`, `topAutoShow`, `multipleItemSelectionAllowed` (`pivottablebuffer.cxx:277-302`).
* Items: `x`, `t` (only `data` processed), `sd`, `h`, `n` (caption; it is read but, per PR #60, *"pivotField name= silently dropped"* and item `n` handling is limited).
* `autoSortScope` / `reference`.

**pivotField: ignored**
* `insertPageBreak`, `hideNewItems`, `includeNewItemsInFilter`, `dragTo*`, `showDropDowns`, `serverField`, `nonAutoSortDefault`, `measureFilter`, `dataSourceSort`, `defaultAttributeDrillState`.
* Item types `default`/`sum`/… (subtotal placeholders) and `grand`/`blank`.
* `x14:pivotField@fillDownLabels` (repeat item labels; per PR #60, this token was absent).

**dataField: read**
* `name`, `fld`, `subtotal` (11 functions), `showDataAs` (9 types), `baseField`, `baseItem`, `numFmtId` (`pivottablebuffer.cxx:1157-1167`). `numFmtId` is used by `testPivotTableValueNumberFormat`.
* If the data field's cache field is a group field, it is redirected to its base (`pivottablebuffer.cxx:1403-1409`).
* Duplicate data fields on the same source field use "Field*" duplicate dimensions (`ScDPUtil::createDuplicateDimensionName`, `dputil.cxx:90`).

**filters**: only `type="count"` (Top-N count) is converted (`pivottablebuffer.cxx:1004-1020`). The full list of ST_PivotFilterType values (caption, value and date filter types) appears in `importPTFilter` (`pivottablebuffer.cxx:968-994`) and is useful as a reference.

### 3.2 Export (`sc/source/filter/excel/xepivotxml.cxx`)

**What is written**
* Cache: `cacheField` name, `numFmtId`, and `sharedItems` with full type flags. The comments at `xepivotxml.cxx:471-513` document Excel's own flag conventions:
  * *"If field contain mixed types (Date and Numbers), MS Excel is saving only "minDate" and "maxDate" and not "minValue" and "maxValue""*, followed by literal Excel examples (`497-501`).
  * `containsSemiMixedTypes` semantics (`471-474`).
  * *"Number type fields could be mixed with blank types, and it shouldn't be treated as listed items."* (`509-513`).
* Records are written using `<x v>` indices. *"That's why in SavePivotCacheXml method, we need to list all items"* (`xepivotxml.cxx:79-80`).
* Date groups (`fieldGroup base`, `rangePr groupBy startDate endDate groupInterval`, `groupItems`).
  * *"Excel seems to expect different order of group item values… output "<date1" first, then elements, then ">date2" last."* (`xepivotxml.cxx:233-237`).
  * *"In Excel, DATE can't be earlier than 1899-12-30 (serial 0) or later than 9999-12-31."* (`314-316`).
* Discrete groups (`discretePr` + `groupItems`) and `fieldGroup par`.
* Calculated fields: `cacheField formula=… databaseField="0"` (`xepivotxml.cxx:715-719`).
* `pivotCacheDefinition createdVersion="3"`: *"MS Excel 2007, tdf#112936: setting version number makes MSO to handle the pivot table differently"* (`xepivotxml.cxx:294`).
* Error and string cache items: *"tdf#123939 : error and string are same for cache; if both are present, keep only one"* (`xepivotxml.cxx:431`).
* Date-time strings are rounded to seconds (tdf#125055, `xepivotxml.cxx:222-228`).
* Table: `pivotTableDefinition` (`dataOnRows`, `dataCaption="Values"` hard-coded, `outline`, `outlineData`, `compact`, `compactData`, `rowGrandTotals`, `colGrandTotals`, `indent=0`, `itemPrintTitles=1`), and `location` (`firstHeaderRow`, `firstDataRow`, `firstDataCol`, `rowPageCount`, `colPageCount`).
* `pivotFields` (`axis`, `dataField`, `compact`, `outline`, `showAll="0"`, subtotal flags, items with `x`/`h`, subtotal items `t=`).
  * tdf#125086 (`xepivotxml.cxx:1475`) covers a field that is both a row field and a data field.
* `rowFields`, `rowItems`, `colFields`, `colItems` (generated from LO's results by `writeRowColumnItems`, `xepivotxml.cxx:1044-1129`, using `t="grand"`, `t="default"` and `r` repeat counts), `pageFields` (`hier=-1` TODO), `dataFields`, `formats`, `pivotTableStyleInfo`.
* dataField `name` is always written: *"tdf#124651: despite being optional in CT_DataField according to ECMA-376 Part 1, Excel (at least 2016) seems to insist on the presence of "name" attribute in dataField element."* (`xepivotxml.cxx:1625-1629`).

**What is dropped**
* **`showDataAs` / `baseField` / `baseItem`.** No hits in the exporter, so show-values-as is lost on xlsx save (tdf#141410, "% of Total not saved for XLSX").
* `sortType` / `autoSortScope`, `autoShow` / top10 filters, captions.
* **Numeric range groups**: only date `rangePr` is written. There is no `startNum`/`endNum` path; `WriteFieldGroup` returns when `!nDatePart` (`xepivotxml.cxx:356-359`).
* Page field selected `item` (third-party report §4).
* `insertBlankRow`, `subtotalTop`, repeat labels, calculated items.
* *"TODO In same cases, disable listing of items, as it is done in MS Excel."* (`xepivotxml.cxx:458-460`).
* *"tdf#126748: DPObject might not reference all group fields… just skip the fields, creating bad cache data, which is of course a temporary hack."* (`xepivotxml.cxx:598-604`).

### 3.3 Legacy xls (BIFF8) (`xipivot.cxx`, `xepivot.cxx`, `xlpivot.cxx`)

These are mostly a parallel implementation. Notable comments:
* `xipivot.cxx:1118-1133`: Excel subtotal names are not escaped; Calc's use `\`.
* `xipivot.cxx:1529-1537`: *"'refresh table on load' flag is set… Some Excel files contain partial table output when this flag is set."*
* `xepivot.cxx:571`: *"Excel writes long indexes even for 0x0100 items"*.
* `xepivot.cxx:1507-1513`: *"Excel expects the records to be filled completely… Excel XP needs a partly initialized SXLI record"*.

---

## 4. Known Excel incompatibilities (code comments + bug trackers)

> bugs.documentfoundation.org and libreoffice.org bugzilla are **not reachable** from this session (DNS blocked). To read them directly, allow those hosts in the session's network settings. The information below comes from search-engine snippets, mailing-list archives (`lists.freedesktop.org` / `mail-archive.com`) and GitHub. The main tracker is **tdf#112416 "[META] XLSX (OOXML) pivot table bugs"**. The general pivot meta bug is tdf#103381, and the XLSX meta bug is tdf#108897.

**Mismatch categories (where Excel's behaviour is subtle):**

1. **Layout forms.**
   * Compact form was long unsupported:
     * tdf#112501 (2017): *"compact mode, which mode is not supported by LO Calc"*; popup buttons misplaced.
     * tdf#155089 / tdf#156207: *"compact view we still can't do it"*; expand icons over text.
   * Before LO 6.0, tabular was exported as outline (tdf#113159).
   * Today LO renders compact per row field, but:
     * compact applies only when `outline && subtotalTop && compact` (`pivottablebuffer.cxx:296`);
     * the compact header row and column offsets were buggy until 2026 (GitHub PR #60, "Alteroffice pivot 2026-05": row labels shifted one column, missing header captions for non-compact fields, `rowHeaderCaption`/`colHeaderCaption` never used, subtotal stripe counting, deep-nesting style cycling mod 3).
   * **Lesson:** Excel's mixed per-field compact/outline/tabular, header-row placement and caption cells are subtle. Use Excel-rendered fixtures.
2. **Captions and labels.** Excel: "Grand Total", "\<item\> Total", "Values", "Sum of X", "Row Labels", "(blank)", "(All)", "(Multiple Items)". LO: "Total Result", "\<item\> Result", "Data", "Sum - X", "(empty)", "- all -", "- multiple -".
   * A docxy issue (GitHub yeroo/docxy #1154, using LO's fixtures) shows Excel re-rendering LO-saved files with "Total Result" → "Grand Total", "Data" → "Values", field name → "Row Labels".
   * tdf#130829: two-row headings where Excel uses one.
3. **"Values" (Σ data-layout) field position.**
   * Excel defaults to columns, and the `-2` field can be placed at any position in rowFields or colFields.
   * LO defaults to ROW when more than one data field is present (`dptabsrc.cxx:922-928`).
   * With one data field, the `-2` field is not written to colFields on export (`xepivotxml.cxx:1205-1206`).
4. **Subtotal placement and functions.**
   * Subtotal-at-top only when a single subtotal row results (`dptabres.cxx:1267-1287`).
   * `defaultSubtotal="0"` was not honoured (PR #60).
   * The XML vs BIFF `defaultSubtotal` semantics differ (`pivottablebuffer.cxx:782-786`).
   * Count/CountA naming is inverted between dataField and pivotField (`pivottablebuffer.cxx:689-693`, `788-794`). COUNT was exported as `count` instead of `countA` (PR #60).
   * tdf#120931: subtotal and grand total rows of Excel pivots were shown as plain values (pivot charts).
5. **Errors in values.**
   * LO always outputs `#VALUE!` (`dpoutput.cxx:473`). Excel shows the actual error, e.g. `#DIV/0!` for calculated fields (docxy #1154: calcfields.xlsx `#VALUE!` → `#DIV/0!`), or `errorCaption` when `showError=1`.
   * Errors in source data poison aggregates except COUNT. Error items in filters: tdf#122471, `testPivotTableErrorItemFilterXLSX/XLSB`.
6. **Date grouping.**
   * Cache group-item order expected by Excel (`xepivotxml.cxx:233-237`). Date filters showed the wrong month (tdf#124736). Exported group fields were misread by Excel 2016 (tdf#124766, still reproducing in 7.2). A grouped-date filter disappeared after re-save (tdf#113908).
   * Ask-LO reports show 13 rows per year (a blank month slot) after Years+Months grouping.
   * Excel 2016+ auto-groups dates on drop (Years/Quarters/Months fields). LO does not.
   * Date clamping: `xepivotxml.cxx:314-316`. 1900 leap-year handling in xlsb (`pivotcachebuffer.cxx:113-119`).
   * tdf#125055: a 1-second difference from datetime rounding.
7. **Show values as.**
   * Not exported to xlsx (tdf#141410).
   * Excel 2010+ types (parent %, rank, % running total) are absent.
   * Previous/next uses the unsorted order (`dptabres.cxx:2694`); verify against Excel.
8. **Calculated fields and items.**
   * Calculated fields were missing until 2025–26 (tdf#126858 *"Calc don't know about Calculated fields in XLSX Pivot table"*, values shown empty). They are now implemented, with SUM-of-fields semantics and `#NAME?` for stale refs (tdf#78486).
   * **Calculated items are still unsupported.**
9. **Stale cache vs live data.** LO refreshes from the sheet on load; Excel shows the cached state (tests `…OutOfSync…`). tdf#160515: formulas referencing pivot cells are wrong after the pivot resizes until a hard recalc.
10. **Filters.**
    * Only top-10 count is imported.
    * Filtering of non-string fields was not exported to XLSX (tdf#107711).
    * Duplicate members in filters (`testPivotTableDuplicatedMemberFilterXLSX`).
    * Page-field selection is lost on re-save (docxy #1154: B3 "mail" → empty).
11. **Robustness.**
    * Exported pivot tables crashed Excel: tdf#108343 (cache corrupted after round trip), tdf#124766, tdf#150954 (pivot gone after re-save), tdf#160310 (errors when re-saving a pivot with a chart).
    * A pivot table is not imported when sheet protection allows pivot editing (tdf#160404).
    * These show which xlsx constraints Excel enforces strictly: dataField `name` required (tdf#124651), `createdVersion` matters (tdf#112936), and items must be listed when records use `<x>`.

---

## 5. Tests and fixtures

### 5.1 Unit tests (`sc/qa/unit/`)

* **`ucalc_pivottable.cxx`** (3530 lines; in-memory engine tests, LO-defined expectations, not oracle):
  * `testPivotTable`, `testPivotTableLabels`, `testPivotTableDateLabels`, `testPivotTableFilters`, `testPivotTableNamedSource`, `testPivotTableCache`.
  * `testPivotTableDuplicateDataFields`, `testPivotTableNormalGrouping`, `testPivotTableNumberGrouping`, `testPivotTableDateGrouping`.
  * `testPivotTableEmptyRows`, `testPivotTableTextNumber` ("0001" text vs numbers), `testPivotTableCaseInsensitiveStrings` ("A" vs "a" → one item), `testPivotTableNumStability` (near-equal doubles from time differences).
  * `testPivotTableFieldReference` (show-values-as), `testPivotTableDocFunc`, `testFuncGETPIVOTDATA`, `testFuncGETPIVOTDATALeafAccess`.
  * `testPivotTableRepeatItemLabels`, `testPivotTableDPCollection`, `testPivotTableMedianFunc`, `testPivotTableUndoCreate`, `testPivotTableFilterValueUndoRedo`.
  * `testPivotTableCalculatedField`, `testPivotTableSpill*` (5), `testPivotTableValueNumberFormat`.
* **`pivottable_filters_test.cxx`** (3123 lines; import/export):
  * Basic, named-range and shared-cache/group ODS.
  * `testGetPivotDataXLS`.
  * Shared group, date-group, nested-date-group and num-group XLSX.
  * `testPivotTableNoColumnsLayout`, `testTdf112501…CompactMode…`, `testPivotTableExportXLSX*`, `testPivotCacheExportXLSX`, `testPivotTableTwoDataFieldsXLSX`, `testPivotTableMedianODS`, `testPivotTableRowHeaderXLS`.
  * Double/String/Date/Bool/RowColPage/Page field filter tests (XLS and XLSX), `testPivotTableEmptyItem`, `testPivotTableFirstHeaderRowXLSX`, `testPivotTableErrorItemFilterXLSX/XLSB`, `testPivotTableErrorItem2FilterXLSX` (tdf122471).
  * Outline/Tabular/Compact layout XLSX, `testPivotTableDuplicatedMemberFilterXLSX`, `testPivotTableDuplicateFields`.
  * `testTdf112106`, `testTdf123923`, `testTdf123939`, `testTdf124651`, `testTdf124736`, `testTdf124772NumFmt`, `testTdf124810*`, `testTdf124883`, `testTdf125046` (long text), `testTdf125055` (1 s difference), `testTdf125086`, `testTdf73845`.
  * `testPivotTableXLSX_OutOfSyncPivotTableCachedDefinitionImport{,2,3}`, `testFirstHeaderRowZero`.
  * `testCalcFields1XLSX/ODS/XLSB`, `testCalcFields2XLSX`, `testGroupAndCalcFieldXLSX`, `testCalcFieldSingleDataDimXLSX`, `testCalcFieldDiffAggregationXLSX` (`pivottable_filters_test.cxx:3035`: COUNT-visible field still uses SUM in the calculated field; values 156/174/330), `testCalcFieldNameErrorXLSX`.
  * `testExtGroupingXLS`, `testPivotOutputPositionXLSB`, `testDatesDiscreteGrouping`, `testInvalidFormats`, `testNumberGroupingXLS`, `testExtSourceFieldsXLS`.
* **`PivotTableFormatsImportExport.cxx`** (pivot area formats): 15 `PivotTableCellFormatsTest_*`, plus `WithCellProtection`, `WithNoSourceData`, `FormatsGrandTotal`, `FormatsSubTotals`, `tdf122473` label rotation.
* **`PivotTable_FieldsAndItemsExport.cxx`** (rowItems/colItems export; compares against Excel-authored files):
  * `testTdf123225*`.
  * `testPivotTable_RowColumnItems_{Row1, Row1_Grand, Row1_Grand_Subtotals, Row2, Row2_Grand, Row2_Subtotals, Row2_Grand_Subtotals, Row2_Compact, Row2_Subtotals_SortDescendingAll, Column3_Grand_Subtotals, Column4_Grand_Subtotals_SortDescending}`.
  * These use `xlsx/pivot/Pivot*.xlsx`, which are Excel-authored. Their `rowItems`/`colItems` encode **Excel's exact item-row enumeration (r/t/x)**. Very useful oracle data for layout.
* `subsequent_export_test*.cxx` / `subsequent_filters_test*.cxx` contain no pivot tests at this commit; they moved to the files above.
* `sc/qa/unit/data/functions/spreadsheet/fods/getpivotdata.fods` is an LO-authored GETPIVOTDATA function test.

### 5.2 Fixtures (`sc/qa/unit/data/…`)

There are 101 matching paths; the full list is in `<scratch>/lo-pivot-fixtures.txt`. All 69 xlsx/xlsb files were downloaded to `<scratch>/fixtures/` (1.3 MB, untrusted data, not opened by any office app). Each was inspected with a read-only zip/regex script (`<scratch>/scripts/inspect.py`) for `docProps/app.xml` Application, `pivotCacheDefinition` `createdVersion`/`refreshedVersion`/`refreshedBy`, and features.

**Excel-authored (last saved by Microsoft Excel or Excel Online; the cache has `refreshedVersion` ≥ 3 and an Excel `refreshedBy`). Candidate free oracle evidence:**

* **Layout and rowItems/colItems (Excel Online 16, refreshedBy "Excel Services"):**
  * `xlsx/pivot/Pivot1_Row.xlsx`, `Pivot1_Row_Grand.xlsx`, `Pivot1_Row_Grand_Subtotals.xlsx`.
  * `Pivot2_Row.xlsx`, `Pivot2_Row_Compact.xlsx` (desktop Excel 16), `Pivot2_Row_Grand.xlsx`, `Pivot2_Row_Grand_Subtotals.xlsx`, `Pivot2_Row_Subtotals.xlsx`, `Pivot2_Row_Subtotals_SortDescendingAll.xlsx`.
  * `Pivot3_Column_Grand_Subtotals.xlsx`, `Pivot4_Column_Grand_Subtotals_SortDescending.xlsx`.
* **Formats (Excel Online 16):**
  * `xlsx/pivot-table/PivotTableCellFormatsTest_{1..15}_*.xlsx`.
  * `PivotTableFormatsGrandTotal.xlsx`, `PivotTableFormatsSubTotals.xlsx`, `first_header_row_zero.xlsx`.
* **Desktop Excel 16:**
  * `xlsx/PivotTable_CachedDefinitionAndDataInSync.xlsx`, `…NotInSync_SheetColumnsRemoved_WithCacheData.xlsx`, `…_WithoutCacheData.xlsx`.
  * `xlsx/pivot-table/pivotcompact.xlsx`, `with-strings-integers-and-dates.xlsx` (mixed types: good for item order), `test_diff_aggregation.xlsx` (calculated field + COUNT), `Pivot_Table_with_Cell_Protection.xlsx`, `PivotTableWithNoSourceData.xlsx` (Excel 2010 cache, refreshed in 16).
  * `xlsx/pivottable_1s_difference.xlsx`, `pivottable_long_text.xlsx`, `tdf89139_pivot_table.xlsx` (page field).
  * `xlsx/pivottable_{bool,date,double,string,rowcolpage}_field_filter.xlsx` (hidden items; caches created by v1, refreshed by v4), `pivottable_duplicated_member_filter.xlsx` (page field), `pivottable_error_item_filter.xlsx`, `pivottable_outline_mode.xlsx`, `pivottable_tabular_mode.xlsx` (page field).
* **Excel 2013 (v15):**
  * `xlsx/pivot-table/shared-dategroup.xlsx`, `shared-group-field.xlsx`, `shared-nested-dategroup.xlsx`, `shared-numgroup.xlsx` (2 tables sharing a cache with groups: **good grouping oracle**).
  * `xlsx/tdf112106.xlsx`.
* **Excel 2010 (v14):**
  * `xlsx/pivot-table/many-fields-in-cache.xlsx`, `pivot_calcfield_nameerror.xlsx` (calculated field), `tdf126858-1.xlsx` (calculated field, Russian), `two-data-fields.xlsx`.
  * `xlsx/pivottable_invalid_formats.xlsx`, `xlsx/tdf122471.xlsx` (error items).
* **Excel 2007 (v12):**
  * `xlsx/pivot_dark1.xlsx`.
  * `xlsb/pivottable_error_item_filter.xlsb`.
* **Excel 16 xlsb:**
  * `xlsb/pivot-table/calcfields.xlsb`.
  * `xlsb/pivot-table/position.xlsb` (Excel Online).

**LO-authored (not oracle):**
* `xlsx/pivot-table/calcfields.xlsx` (Collabora 26.04), `groupwithcalcfields.xlsx` and `onlycalcfields.xlsx` (LibreOfficeDev 26.8), `tdf122473_PivotTableLabelRotation.xlsx` (LO 7.6), `xlsx/pivot_table_first_header_row.xlsx` (LODev 6.0).
* All `ods/*` and `.fods`: `pivot-table-basic`, `-err-in-cache`, `-median`, `-named-range-source`, `-num-fmt`, `-shared-cache-with-group`, `-str-and-err-in-data`, `pivottable_fieldInRowsAndData`, `pivottable_no_columns_layout`, `pivottable_restore_query_filter`, `tdf123225_*` (3), `tdf123421_{1,2}datafield(s)`, `tdf124651_simplePivotTable`.
* docxy #1154 shows that Excel re-renders some of these differently. That makes them useful **negative** examples, but only once Excel's output is captured.

**Probably Excel-authored, not verified (BIFF8; app info is in the OLE SummaryInformation, which was not checked):**
* `xls/pivot-getpivotdata.xls`, `pivot_row_header.xls`, `pivottable_{bool,date,double,string,rowcolpage,page}_field_filter.xls`, `pivottable_empty_item.xls`.
* `pivottable_dates_grouping.xls`, `pivottable_ext_grouping.xls`, `pivottable_ext_source_fields.xls`, `pivottable_number_grouping.xls`, `tdf112501.xls`.
* `xls/pass/forcepoint-pivot-1.xls` (fuzzer crash input: useful for robustness, not values).

**Caveats for using these as oracle evidence.**
* "Last saved by Excel" means the rendered cells and `rowItems` are Excel's **as of the last refresh**. A file can be saved without refresh; check `refreshOnLoad` and compare records against the sheet. The "NotInSync" fixtures are deliberately stale.
* Feature coverage visible in the pivotTable XML: none of the fixtures use `showDataAs`, calculated items, or x14 show-as. Several use `sortType`, `pageField`, compact=0 / outline=0, and calculated fields (in the cache XML).
* Licence: the fixtures sit in the MPL-2.0 LibreOffice repo, but some are user-submitted bug attachments. Check provenance per file before vendoring. Extracting values as derived test expectations is probably safer than copying the files.

---

## 6. Takeaways for RustSheet

1. **Use Excel's own rendered output as the oracle.** That means the cells plus `rowItems`/`colItems`/`location`. Do not, like LO, regenerate on import and lose fidelity. A clean design: compute the result tree, then emit `rowItems`/`colItems`, then render. Diff both against the Excel fixtures (the `xlsx/pivot/Pivot*.xlsx` set is ideal for the item-enumeration algorithm).
2. **Probe list** (the LO choices that need Excel evidence):
   * Case-insensitive item dedup and which casing wins.
   * Type order of items, including booleans and errors.
   * Error propagation in aggregates and which error is displayed (`showError` / `errorCaption`).
   * Text in Sum/Avg; empty Average shown blank vs `#DIV/0!`.
   * StdDev of one value.
   * Top-N ties; whether totals include Top-N-hidden items.
   * Sort-by-value tie stability and the error position.
   * Previous/next base item under a custom sort.
   * Running-total grand total blank.
   * Reference-item difference blank.
   * The various `#N/A`-style results for show-values-as.
   * Numeric group label rules (integer vs fractional; last group).
   * Day grouping of Feb-29.
   * Date-group labels.
   * Subtotal-at-top with multiple data fields.
   * Row-forced vs column-forced subtotal intersections.
   * Page field captions.
   * Duplicate header renaming.
   * Calculated-field semantics at totals (formula on sums) and non-measure references (0).
   * GETPIVOTDATA matching (number vs text, dates, ambiguity, hidden items).
3. **Gaps where LO gives no help:** calculated items, Excel 2010+ show-values-as (parent %, rank, % running total), label/value/date filters, "include filtered items in totals" (OLAP-only in Excel), `visualTotals`, auto date grouping, and Excel's exact error display.
4. **Directive 5.** LO caps output by sheet limits (`mbSizeOverflow`, `SC_MINCOUNT_LIMIT`, `dptabsrc.cxx:993-1003`). RustSheet needs explicit item, row and column caps, plus recursion limits for calculated fields (LO uses a recursion-stack set for cycle detection).

## Sources (web)
- [GitHub LibreOffice/core PR #60 "Alteroffice pivot 2026 05"](https://github.com/LibreOffice/core/pull/60)
- [yeroo/docxy issue #1154: pivot tables change after an unedited xlsx save](https://github.com/yeroo/docxy/issues/1154)
- [tdf#112501 (libreoffice-bugs archive)](https://lists.freedesktop.org/archives/libreoffice-bugs/2017-September/041539.html), [tdf#113159](https://lists.freedesktop.org/archives/libreoffice-bugs/2017-October/049193.html), [tdf#156207](https://www.libreoffice.org/bugzilla/show_bug.cgi?id=156207)
- [tdf#126858 calculated fields in XLSX](https://lists.freedesktop.org/archives/libreoffice-bugs/2019-August/208523.html), [tdf#120931](https://www.libreoffice.org/get-help/bug/show_bug.cgi?id=120931), [tdf#141410](https://www.libreoffice.org/get-help/bug/show_bug.cgi?id=141410)
- [tdf#124736](https://lists.freedesktop.org/archives/libreoffice-bugs/2019-April/181503.html), [tdf#124766](https://www.libreoffice.org/bugzilla/show_bug.cgi?id=124766), [tdf#107711](https://www.libreoffice.org/bugzilla/show_bug.cgi?id=107711)
- [tdf#112416 META XLSX pivot bugs (mail archive)](https://www.mail-archive.com/libreoffice-bugs@lists.freedesktop.org/msg1073291.html), [tdf#160404](https://www.mail-archive.com/libreoffice-bugs@lists.freedesktop.org/msg1014652.html), [tdf#160310](https://www.mail-archive.com/libreoffice-bugs@lists.freedesktop.org/msg1013792.html), [tdf#160515](https://www.mail-archive.com/libreoffice-bugs@lists.freedesktop.org/msg1015925.html), [tdf#150954](https://www.mail-archive.com/libreoffice-bugs@lists.freedesktop.org/msg891232.html)
- [Ask LibreOffice: date grouping breaks on reload](https://ask.libreoffice.org/t/grouping-pivot-table-dates-by-month-and-year-breaks-on-reload/136742)
