# Excel PivotTables & PivotCharts: feature catalog from public Microsoft documentation

> Research spike of 2026-11-04 (see plan.md in this folder). `<scratch>/` paths were session scratch files and are not committed. Hypotheses here are not Excel evidence until an oracle probe confirms them.

Desk research for RustSheet, done 2026-11-04. It extends SPEC-0015, which covers read-only import plus GETPIVOTDATA.
Directive 1 still applies: **nothing here is oracle evidence**. Use this catalog to plan probes, not to decide behavior.

## How to read this file

### Evidence tags

| Tag | Meaning |
|---|---|
| **[D]** | Stated in Microsoft prose: support.microsoft.com, learn.microsoft.com, or the Microsoft 365 blogs. The URL is given. |
| **[S]** | Comes from the schema. Either the ECMA-376 transitional XSD (`sml.xsd`, `dml-chart.xsd`), the [MS-XLSX] text embedded in the Open XML SDK pages, or the [MS-OI29500] Office implementation notes. |
| **[K]** | Background knowledge that no page fetched in this session confirms. Treat it as **unverified** and probe it before relying on it. |

### Column meanings

- **Values?**
  - **V**: changes computed cell values or which cells exist.
  - **L**: changes layout only, meaning where values and labels land. It has no effect on the numbers themselves.
  - **F**: affects formatting or visuals only.
  - **M**: metadata or UI-only.
- **Web**: whether Excel for the web supports the feature. The values are **yes**, **no**, **view** (renders an existing pivot but cannot create one), or **?** (unknown).
- **JS**: whether Office.js (the Excel JavaScript API) can create or set the feature.
- **Graph**: whether Microsoft Graph can do so. In practice Graph can only **list, get and refresh** pivots (see section 4).

### Abbreviations

| Abbreviation | Meaning |
|---|---|
| PTD | `pivotTableDefinition` (xl/pivotTables/pivotTableN.xml) |
| PCD | `pivotCacheDefinition` |
| PCR | `pivotCacheRecords` |
| `pf` | `pivotField` |
| `cf` | `cacheField` |
| `df` | `dataField` |

### Primary references used throughout

- ECMA-376 transitional schema, `sml.xsd`, for SpreadsheetML §18.10. Mirror: https://raw.githubusercontent.com/python-openxml/python-docx/master/ref/xsd/sml.xsd (lines 555-1720 are the pivot types).
- `dml-chart.xsd`, for the chart elements (`c:pivotSource`, `c:pivotFmts`). Mirror: https://raw.githubusercontent.com/python-openxml/python-docx/master/ref/xsd/dml-chart.xsd
- [MS-OI29500] Office implementation notes, §18.10.*. These are the most useful "what Excel actually does" notes.
  - Index: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/ (TOC entries 2.1.737–2.1.803)
  - dataField note: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/0736556b-ddde-42e4-ba59-40410c5735c2
  - Pivot Formulas note: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/b23ed080-d4e1-43a5-8259-332e1bf85d08
- Open XML SDK class reference. It also embeds the [MS-XLSX] text for the x14/x15 elements. Examples:
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.pivottabledefinition
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2010.excel.pivottabledefinition
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2010.excel.datafield
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2013.excel.datafield
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2010.excel.slicercachedefinition
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2010.excel.tabularslicercache
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2013.excel.timelinestate
  - https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.office2010.drawing.charts.pivotoptions
- Office.js:
  - Overview: https://learn.microsoft.com/en-us/office/dev/add-ins/excel/excel-add-ins-pivottables
  - Class pages: https://learn.microsoft.com/en-us/javascript/api/excel/excel.pivottable, `excel.pivotlayout`, `excel.pivotfield`, `excel.datapivothierarchy`, `excel.showascalculation`, `excel.pivotvaluefilter`, `excel.datefiltercondition`, `excel.labelfiltercondition`, `excel.subtotals`
- Microsoft Graph:
  - v1.0: https://learn.microsoft.com/en-us/graph/api/resources/workbookpivottable?view=graph-rest-1.0
  - beta: https://learn.microsoft.com/en-us/graph/api/resources/workbookpivottable?view=graph-rest-beta

Short names used for the support.microsoft.com articles cited throughout:

| Short name | Article |
|---|---|
| CALC | "Calculate values in a PivotTable": https://support.microsoft.com/en-us/office/calculate-values-in-a-pivottable-11f41417-da80-435c-a5c6-b0185e59da77 |
| SVA | "Show different calculations in PivotTable value fields": https://support.microsoft.com/en-us/office/show-different-calculations-in-pivottable-value-fields-014d2777-baaf-480b-a32b-98431f48bfec |
| SUMFN | "Change the summary function or custom calculation for a field in a PivotTable": https://support.microsoft.com/en-us/excel/change-the-summary-function-or-custom-calculation-for-a-field-in-a-pivottable |
| LAYOUT | "Design the layout and format of a PivotTable": https://support.microsoft.com/en-us/office/design-the-layout-and-format-of-a-pivottable-a9600265-95bf-4900-868e-641133c05a80 |
| OPTS | "PivotTable options": https://support.microsoft.com/en-us/excel/pivottable-options |
| TOTALS | "Subtotal and total fields in a PivotTable": https://support.microsoft.com/office/subtotal-and-total-fields-in-a-pivottable-173f5b30-b546-4293-87d2-aee638f74e7d |
| FILTER | "Filter data in a PivotTable": https://support.microsoft.com/en-us/topic/cc1ed287-3a97-4e95-b377-ddfafe79fa8f |
| SORT | "Sort data in a PivotTable or PivotChart": https://support.microsoft.com/en-us/Excel/sort-data-in-a-pivottable-or-pivotchart |
| GROUP | "Group or ungroup data in a PivotTable": https://support.microsoft.com/en-us/office/group-or-ungroup-data-in-a-pivottable-c9d1ddd0-6580-47d1-82bc-c84a5a340725 |
| REFRESH | "Refresh PivotTable data": https://support.microsoft.com/office/refresh-pivottable-data-6d24cece-a038-468a-8176-8b6568ca9be2 |
| CREATE | "Create a PivotTable to analyze worksheet data": https://support.microsoft.com/en-us/office/create-a-pivottable-to-analyze-worksheet-data-a9a84538-bfe9-40a9-a8e9-f99134456576 |
| GPD | "GETPIVOTDATA function": https://support.microsoft.com/en-us/office/getpivotdata-function-8c083b99-a922-4ca0-af5e-3af55960761f |
| OVERVIEW | "Overview of PivotTables and PivotCharts": https://support.microsoft.com/en-us/office/overview-of-pivottables-and-pivotcharts-527c8fa3-02c0-445a-a2db-7794676bce96 |
| PCHART | "Create a PivotChart": https://support.microsoft.com/en-us/office/create-a-pivotchart-c1b1e057-6990-4c38-b52b-8255538e7b1c |
| SLICER | "Use slicers to filter data": https://support.microsoft.com/en-us/office/use-slicers-to-filter-data-249f966b-a9d5-4b0f-b31a-12651785d29d |
| TIMELINE | "Create a PivotTable timeline to filter dates": https://support.microsoft.com/en-US/excel/create-a-pivottable-timeline-to-filter-dates |
| DETAILS | "Expand, collapse, or show details in a PivotTable or PivotChart": https://support.microsoft.com/en-au/office/expand-collapse-or-show-details-in-a-pivottable-or-pivotchart-d70d7e70-d230-4d45-81db-1f5e39bcb394 |
| WEBDIFF | "Differences between using a workbook in the browser and in Excel": https://support.microsoft.com/en-us/office/differences-between-using-a-workbook-in-the-browser-and-in-excel-f0dc28ed-b85d-4e1d-be6d-5878005db3b6 |
| CALCITEMS-SVC | "Calculated items in Excel and Excel Services": https://support.microsoft.com/office/calculated-items-in-excel-and-excel-services-2d33ff89-38f7-4ed9-9386-7b7b4f3c151a |
| LIMITS | "Excel specifications and limits": https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3 |
| AUTOREFRESH | Insider blog, "Keep data updated with PivotTable Auto Refresh in Excel": https://techcommunity.microsoft.com/blog/microsoft365insiderblog/keep-data-updated-with-pivottable-auto-refresh-in-excel/4436409 |
| PSPILL | Insider blog, "Stay in the flow with PivotTable #SPILL in Excel": https://techcommunity.microsoft.com/blog/microsoft365insiderblog/stay-in-the-flow-with-pivottable-spill-in-excel/4458201 |

---

## 0. Headline findings (read first)

1. **The Graph oracle cannot build pivots.** The Graph `workbookPivotTable` resource has only `id`, `name` and `worksheet`. Its only methods are **list, get, refresh, and refreshAll** (refreshAll exists only on the collection). This is identical in v1.0 and beta [D: Graph pages above].
   - Graph has no API for fields, layout, filters, data fields or show-as.
   - **Every pivot probe must therefore be an authored xlsx.** The probe file carries a PTD and a PCD. It can also carry PCR, or set `refreshOnLoad="1"`.
   - The probe workflow is:
     1. Upload the xlsx to OneDrive.
     2. Call `POST …/worksheets/{id}/pivotTables/{id}/refresh`, or `…/pivotTables/refreshAll`.
     3. Read the rendered cells with the normal `range` / `usedRange` endpoints. GETPIVOTDATA formulas are read the same way.
   - Office.js can create pivots and set most layout and data options, but it is not reachable from Graph. Office Scripts use the same object model and run in Excel for the web, but they are reached through Power Automate, not Graph. They are an alternative oracle channel if one is ever needed. **[K]** No Graph endpoint to run an Office Script was found.
2. **Office.js has a large surface but leaves important gaps** (section 4):
   - Missing: grouping (numeric, date, manual), calculated fields and calculated items, Distinct Count, OLAP and Data Model, PivotCharts, "show items with no data" per se, number-of-items-to-retain, and pivot style assignment.
   - Present: summarizeBy (11 functions), all 14 ShowAs calculations, label/value/date/manual filters, layout type, subtotals, grand totals, repeat labels, blank rows, empty-cell text, and sorting.
3. **Excel for the web** can create pivots and edit fields, Summarize-by, Show-Values-As, sort, filter, local-pivot slicers and the settings pane [D: CREATE, FILTER, SLICER, REFRESH].
   - Community and Microsoft Q&A answers (not official docs) say the web **cannot create calculated fields or grouping**.
   - It does render and recalculate existing calculated fields: "once it's created, it will continue to work and display properly in Excel for the web". Source: https://learn.microsoft.com/en-us/answers/questions/5918589/excel-web-pivot-table-calculations.
   - The web has no Data Model creation [D: CREATE (web section has no Data Model option); SLICER: "only local PivotTable slicer creation"].
   - Whether a Graph `refresh` on an authored file with `fieldGroup` or `calculatedItem` recomputes correctly in the web engine is **unknown**. Probe this first.
4. **Two new behaviors from 2025 change the model:**
   - **PivotTable Auto Refresh.** This is a per-source setting, **on by default for new pivots on local data**. Insider-only per REFRESH and AUTOREFRESH.
   - **PivotTable #SPILL!** A blocked pivot shrinks to one cell holding `#SPILL!`. This affects GETPIVOTDATA and PivotCharts, while slicers and timelines are unaffected. Beta Channel, Windows 2508 and Mac 16.102 [D: PSPILL].
   - Neither is mentioned for the web. **[K]** Neither feature has a documented attribute in the file format.
5. **The richest "what Excel really does" source is [MS-OI29500]**, which has 67 notes on §18.10. Several of them invert the standard:
   - `asteriskTotals=1` means asterisks are **not** shown.
   - `preserveFormatting=1` means **preserve**.
   - `item@s` marks a **manual filter**, not "has string".
   - `item@sd` means **expanded**.
   - `indent` is one less than the number of increments, and 127 means no indent.
   - `chartFormat@format` indexes `c:pivotFmt`, not a dxf.

   Section 7 lists all of them.

---

## 1. Feature catalog

### 1.1 Source & cache

| Feature | One-line description | OOXML storage (defaults) | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Worksheet range source | Pivot built from an A1 range (needs a header row of unique, non-blank labels, one type per column) | PCD `<cacheSource type="worksheet">` + `<worksheetSource ref= sheet=>`; Office requires exactly one of `ref`/`name` [S OI 18.10.1.95] | V | yes | `pivotTables.add(name, source, dest)` (source = range addr/Range) | no | CREATE; JS overview |
| Table / defined-name source | Source is an Excel table or a named range; table growth is picked up on refresh | `worksheetSource name="Table1"` (≤255 chars) | V | yes | `add(name, Table, dest)`; `getDataSourceType()` → LocalRange/LocalTable/Unknown, `getDataSourceString()` (ExcelApi 1.15) | no | CREATE; JS PivotTable page |
| External source (connection, ODBC, Power Query) | Cache filled from a workbook connection | `cacheSource type="external" connectionId=N` (N = `connection@id`, >0) [S OI 18.10.1.7]; `backgroundQuery` must match connection | V | partial ("some kinds of data connections are supported") | no | no | WEBDIFF |
| Multiple consolidation ranges | Wizard (Alt+D+P) pivot over several ranges; produces generic Row/Column/Value/Page1..4 fields | `cacheSource type="consolidation"` → `<consolidation autoPage=true>` `<pages>` (1–4 `page`, each `pageItem name`) + `<rangeSets>` `<rangeSet i1..i4 ref sheet name r:id>` | V | ? (probably view only) | no | no | [S] sml.xsd; OI 18.10.1.79; field names [K] |
| Scenario source | Scenario Manager summary pivot | `cacheSource type="scenario"` | V | ? | no | no | [S] |
| Cache sharing | Pivots built from the same source in one workbook share one PCD. Refresh, group/ungroup and calculated fields/items affect all of them | PTD `cacheId` → workbook `<pivotCaches><pivotCache cacheId r:id>` | V | yes | no | no | OVERVIEW ("Shared-cache effects") |
| Cache fields | One per source column (+ calculated fields, + group fields) | `<cacheFields count>` `<cacheField name! caption numFmtId formula databaseField=true serverField=false uniqueList=true sqlType=0 hierarchy=0 level=0 mappingCount memberPropertyField=false>`; name unique, case-insensitive, 1..32767 chars; ≤16384 fields (≤1024 if createdVersion<3); first field must be `databaseField=1` [S OI 18.10.1.3/.4] | V | yes | `hierarchies` (1 hierarchy = 1 field) | no | |
| Shared items | Distinct values of a field, typed | `<sharedItems>` children `m` (missing/blank), `n v=double`, `b v`, `e v="#N/A"`, `s v`, `d v=dateTime`; flags `containsSemiMixedTypes=1 containsNonDate=1 containsDate=0 containsString=1 containsBlank=0 containsMixedTypes=0 containsNumber=0 containsInteger=0 minValue maxValue minDate maxDate count longText=0`. Excel: bool and error count as "string" for containsString/containsMixedTypes; containsInteger = only integers; ≤1,048,576 items (32,500 if createdVersion<3); NaN/INF forbidden | V | — | — | — | [S] sml.xsd; OI 18.10.1.90/.60 |
| Fields without shared items | Numeric-only value fields are often written with an empty `<sharedItems>` (+ min/max) and records store the value inline (`<n v>`) instead of an index (`<x v>`) | PCR `<r>` children: `x v=index` or inline `m/n/b/e/s/d` | V | — | — | — | [S]; usage pattern [K] |
| Cache records | Snapshot of source rows | `pivotCacheRecords` part (`<r>` per record; child count = # cacheFields with `databaseField=1`) [S OI 18.10.1.77]; `recordCount` on PCD | V | — | — | — | [S] |
| Save source data with file | If off, no PCR part; a refresh is needed to recompute | PCD `saveData=true` | V (no records → nothing to compute without refresh) | ? | no | no | OPTS "Data"; SPEC-0015 unknown 12 |
| Refresh on open | Re-read source when workbook opens | PCD `refreshOnLoad=false` | V | yes ("Refresh data on file open" in settings pane) | `PivotTable.refreshOnOpen` (ExcelApi 1.13) | no | REFRESH; JS |
| Number of items to retain per field | Keep deleted source items in the cache (Automatic/None/Max) | PCD `missingItemsLimit` (absent = automatic; 0 = none; 1048576 = max) | V (deleted items still listed with "show items with no data", filters, slicers) | ? | no | no | OPTS "Data"; mapping [K] |
| Invalid / refresh-disabled cache | Marks stale cache, disallows refresh | PCD `invalid=false`, `enableRefresh=true`, `upgradeOnRefresh=false`, `refreshedVersion`, `createdVersion`, `minRefreshableVersion` | M | — | — | — | [S] |
| Optimize memory / background query | External-source tuning | PCD `optimizeMemory=false`, `backgroundQuery=false` (must be 0 for non-external) | M | — | — | — | OI 18.10.1.67 |
| Data Model / Power Pivot source | Pivot over the in-workbook Data Model (OLAP-like cube); enables Distinct Count, DAX measures, relationships | PCD `cacheSource type="external"` to the `ThisWorkbookDataModel` connection, `cacheHierarchies`, `x15:pivotCacheDecoupled`, `x15:dataModel`, `xl/model/item.data` | V | view/refresh only; no creation [D: CREATE web, SLICER] | no ("no support for Power Pivot") | no | JS overview "Important"; **out of scope candidate** |
| OLAP source (SSAS cube) | Server-calculated pivot; summary functions not changeable | `cacheHierarchies`, `kpis`, `tupleCache`, `calculatedMembers`, `dimensions`, `measureGroups`, `maps`; PTD `pivotHierarchies`, `rowHierarchiesUsage`, `colHierarchiesUsage` | V (server-computed) | view/edit, not create [D: CALC web] | no ("PivotTables created with OLAP are not currently supported") | no | CALC "OLAP source data"; **out of scope** |

### 1.2 Layout & rendering

| Feature | Description | OOXML (defaults) | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Location | The rendered rectangle excluding page fields; header/data offsets | PTD `<location ref! firstHeaderRow! firstDataRow! firstDataCol! rowPageCount=0 colPageCount=0>`; Office: `ref` excludes page fields; `firstHeaderRow ≤ firstDataRow` | L | — | `layout.getRange()` (excl. filter area), `getFilterAxisRange`, `getRowLabelRange`, `getColumnLabelRange`, `getDataBodyRange` (ExcelApi 1.8) | no (but Graph `range` reads cells) | OI 18.10.1.49; JS PivotLayout |
| Areas: Rows / Columns / Values / Filters | A field can sit in only one of rows, columns or filters. A field can also be used as a value, more than once | `pf@axis` ∈ axisRow/axisCol/axisPage (absent = not on an axis; axisValues not allowed); `pf@dataField=1` when used as value; `<rowFields><field x>`, `<colFields>`, `<pageFields><pageField fld item hier=-1 name cap>`, `<dataFields><dataField name fld! subtotal=sum showDataAs=normal baseField=-1 baseItem=1048832 numFmtId>` | V | yes (field list) | row/column/filter/dataHierarchies `.add/.remove`; position | no | JS overview "Hierarchies"; OI 18.10.1.69 |
| "Values" pseudo-field (Σ Values) | With ≥2 data fields, a virtual field (index **-2**) placed on rows or columns | `<field x="-2"/>` in rowFields/colFields; PTD `dataOnRows=false`, `dataPosition` (default 0 per OI) | L | yes | (implicit) | no | OI 18.10.1.29 (x = -2 allowed) |
| Row/col items encoding | Pre-rendered axis lines | `<rowItems>/<colItems>` of `<i t=data r=0 i=0><x v=0/>…</i>`; `r` = number of leading fields repeated from previous line; `i` = data-field index; `x@v` = index into `pf/items` (or into dataFields when the field is -2); `v=1048832` = no item; `t` ∈ data, default (subtotal), sum…varP (custom subtotal), grand, blank (blank row) | L | — | — | — | [S]; OI 18.10.1.44 |
| Report layout: Compact (default) | All row fields in one column, indented; "Row Labels" header | PTD `compact=true outline=false compactData=true outlineData=false`; per field `pf compact=true outline=true` | L | yes | `layout.layoutType` = Compact/Outline/Tabular (null if mixed) | no | LAYOUT; JS |
| Outline form | One column per row field; subtotals may be at top | PTD `outline=true`; `pf outline=true compact=false` | L | yes | yes | no | LAYOUT |
| Tabular form | One column per field, subtotals at bottom, field headers | `pf outline=false compact=false` | L | yes | yes | no | LAYOUT |
| Per-field compact label display | "Display labels from the next field in the same column" | `pf@compact` | L | ? | no | no | LAYOUT |
| Indent | Compact-form row label indent 0..127 | PTD `indent=1` (Office: value is one less than the number of increments; 127 = none) | F | ? | no | no | OPTS; OI 18.10.1.73 bb |
| Repeat item labels (fill down) | Repeat outer labels on every row (outline/tabular only) | `x14:pivotField fillDownLabels=false` (ignored when field compact && outline); PTD ext `x14:pivotTableDefinition fillDownLabelsDefault=false` | L (cells become non-empty) | yes ("Repeat/Don't repeat") | `layout.repeatAllItemLabels(bool)` (1.13) | no | LAYOUT; MS-XLSX via SDK |
| Blank row after each item | Insert an empty line after each outer item | `pf insertBlankRow=false`; rendered as `<i t="blank">` | L | yes | `layout.displayBlankLineAfterEachItem(bool)` (1.13) | no | LAYOUT |
| Insert page break after each item | Print layout | `pf insertPageBreak=false` | F | no | no | no | [S] |
| Field headers / captions | "Row Labels"/"Column Labels" headers and filter buttons | PTD `showHeaders=true`, `rowHeaderCaption`, `colHeaderCaption` (absent = app string), `dataCaption!` (e.g. "Values"), `showDropZones`, `showDataDropDown` | L (header cell text) | yes ("Field captions and filters") | `layout.showFieldHeaders` (1.13) | no | OPTS Display; FILTER |
| Classic layout (drag in grid) | 2003-style drop zones | PTD `gridDropZones=false` (also implies outline layout) [K] | L | no | no | no | OPTS |
| Merge & center labels | Merge outer label cells | PTD `mergeItem=false` | F | ? | no | no | OPTS |
| Expand/collapse items | Collapse an outer item to its subtotal row | `item@sd=true` (Office: `sd`=expanded; false hides details) | L (child rows disappear; subtotal remains) | yes | `PivotItem.isExpanded` | no | OI 18.10.1.45 l; DETAILS |
| Show expand/collapse buttons | ± buttons | PTD `showDrill=true`, `printDrill=false` | F | ? | no | no | OPTS |
| Values row | Header row labeling multiple value fields | `x14:pivotTableDefinition hideValuesRow=false` | L | ? | no | no | OPTS ("Show the Values row"); MS-XLSX |
| Empty-cell text | Text for empty data cells | PTD `showMissing=true`, `missingCaption` (absent + showMissing = blank) | V/L (cell content: blank vs text) | yes | `layout.fillEmptyCells` (default false), `layout.emptyCellText` (1.13) | no | OPTS; LAYOUT; JS |
| Error-cell text | Replace error results with text | PTD `showError=false`, `errorCaption` | V (cell content) | yes ("Error values as") | no | no | OPTS; LAYOUT |
| Grand-total caption | Custom "Grand Total" label | PTD `grandTotalCaption` | L | ? | no | no | [S] |
| Subtotal caption | Custom subtotal label; `?` → field name | `pf subtotalCaption` | L | ? | no | no | OI 18.10.1.69 bb |
| Item captions (renamed items) | Rename an item's label in place | `item@n` (unique within field, ≤32767) | L (label text, GETPIVOTDATA matching [K]) | ? | `PivotItem.name` (settable [K]) | no | OI 18.10.1.45 q,r |
| Field/data-field captions | "Sum of Sales" etc. renamable | `pf@name`, `df@name` | L | yes | `DataPivotHierarchy.name`, `PivotField.name` | no | JS overview "Change hierarchy names" |
| Display item labels with no values | Show axis labels when Values area empty | PTD `showItems=true` [K mapping] | L | ? | no | no | OPTS |
| Show items with no data (non-OLAP) | List all cache items of a field even if no records match current filters | `pf showAll=true` per XSD, but Excel usually writes `showAll="0"`; OLAP uses PTD `showEmptyRow/showEmptyCol` (OLAP only) | V/L (extra rows of blanks) | ? | `PivotField.showAllItems` (1.8) | no | OI 18.10.1.69 w, 18.10.1.73 ee/ff; OPTS (OLAP-only for table-wide option) |
| Report filter area arrangement | Down-then-over / over-then-down, N per column | PTD `pageOverThenDown=false`, `pageWrap=0` (≤255); `location rowPageCount colPageCount` | L | ? | no | no | OPTS; FILTER |
| Alt text | Accessibility title/description | `x14:pivotTableDefinition altText altTextSummary` (≤2000 chars) | M | ? | `layout.altTextTitle/altTextDescription` | no | MS-XLSX via SDK; JS |
| Pivot #SPILL! | When the pivot cannot grow (blocked by text/merged cells/tables/sheet edge) it collapses to one cell with `#SPILL!` | (no documented attribute) | V | ? | ? | ? | PSPILL (Beta 2508+) |
| Overlap rules | A pivot can't overlap another pivot/table (classic error dialog) | — | V | — | — | — | [K] |

### 1.3 Aggregation (Summarize Values By)

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Sum | Default for numeric value fields | `df subtotal="sum"` (default) | V | yes | `summarizeBy = Sum` | no | SUMFN |
| Count | Counts non-empty values, like COUNTA. Default for non-numeric **or blank-containing** value fields | `subtotal="count"` | V | yes | Count | no | SUMFN; CALC ("Count is the default for non-numeric or blank value fields") |
| Average, Max, Min, Product | — | `average`, `max`, `min`, `product` | V | yes | yes | no | SUMFN |
| Count Numbers | Like COUNT | `countNums` | V | yes | CountNumbers | no | SUMFN |
| StdDev / StdDevp / Var / Varp | Sample and population | `stdDev`, `stdDevp`, `var`, `varp` | V | yes | yes | no | SUMFN |
| Automatic | API-only pseudo-function (Sum for numbers else Count) | — | V | — | `Automatic` (default) | no | JS overview |
| Distinct Count | Unique values; **Data Model only** | `x15:dataField isCountDistinct` (ext on dataField) + Data Model cache | V | no (Data Model) | no | no | SUMFN; MS-XLSX via SDK |
| Same field as several values | Add a field twice ("Sum of X", "Count of X2"/"_2" suffix) | multiple `df` with same `fld` | V | yes | `dataHierarchies.add` again | no | SVA notes; SUMFN |
| Value number format | Number format of a data field | `df numFmtId` (absent = app default) | F (value unchanged; display) | yes (Home number format) | `DataPivotHierarchy.numberFormat` (1.8) | no | SUMFN; OI 18.10.1.22 d |
| Field number format (labels) | Number format of row/col item labels | `pf numFmtId` (default 0) | F | ? | no | no | LAYOUT; OI 18.10.1.69 c |
| Cache-field number format | Source format carried into the cache | `cf numFmtId` | F | — | — | — | [S] |
| Restriction | Summary function can't be changed for OLAP data, calculated fields, or fields with calculated items | — | V | — | — | — | SUMFN |

### 1.4 Show Values As (custom calculations)

ECMA `df@showDataAs` (ST_ShowDataAs) has 9 values: `normal`, `difference`, `percent`, `percentDiff`, `runTotal`, `percentOfRow`, `percentOfCol`, `percentOfTotal`, `index`.

The Excel 2010+ modes live in `x14:dataField pivotShowAs` (ST_PivotShowAs). Its values are `percentOfParent`, `percentOfParentRow`, `percentOfParentCol`, `percentOfRunningTotal`, `rankAscending` and `rankDescending`. When `pivotShowAs` is present, `showDataAs` **must not** be written. For percentOfParent, percentOfRunningTotal and the two rank modes, `baseField ≥ 0` is required [S: MS-XLSX x14 dataField via SDK page].

| UI name (SVA, SUMFN) | Formula as documented | OOXML | Base field / item | JS `ShowAsCalculation` | Sources |
|---|---|---|---|---|---|
| No Calculation | value | `showDataAs="normal"` | — | `none` | SVA |
| % of Grand Total | value / grand total | `percentOfTotal` | — | `percentOfGrandTotal` | SVA |
| % of Column Total | value / column total | `percentOfCol` | — | `percentOfColumnTotal` | SVA |
| % of Row Total | value / row total | `percentOfRow` | — | `percentOfRowTotal` | SVA |
| % Of | value / value of base item in base field | `percent` + `baseField`, `baseItem` | field + item, or (previous)/(next) | `percentOf` | SVA; OI 18.10.1.22 e,f |
| % of Parent Row Total | value / value of parent item on rows | x14 `percentOfParentRow` | — | `percentOfParentRowTotal` | SVA |
| % of Parent Column Total | value / value of parent item on columns | x14 `percentOfParentCol` | — | `percentOfParentColumnTotal` | SVA |
| % of Parent Total | value / value of parent item of the selected base field | x14 `percentOfParent` + baseField | field | `percentOfParentTotal` | SVA; CREATE web ("hover over the option and select the parent field") |
| Difference From | value − base item value | `difference` + baseField/baseItem | field + item, or (prev)/(next) | `differenceFrom` | SVA; JS (missing base → **#N/A**, see section 2) |
| % Difference From | (value − base) / base | `percentDiff` + baseField/baseItem | field + item, or (prev)/(next) | `percentDifferenceFrom` | SVA |
| Running Total In | cumulative over successive items of the base field | `runTotal` + baseField | field | `runningTotal` | SVA; OI 18.10.1.22 e |
| % Running Total In | running total / total | x14 `percentOfRunningTotal` + baseField | field | `percentRunningTotal` | SVA |
| Rank Smallest to Largest | rank 1 = smallest | x14 `rankAscending` + baseField | field | `rankAscending` | SVA |
| Rank Largest to Smallest | rank 1 = largest | x14 `rankDescending` + baseField | field | `rankDecending` (sic) | SVA |
| Index | (value × grand total of grand totals) / (grand row total × grand column total) | `index` | — | `index` | SVA; JS enum doc |

Further rules for Show Values As:

- **Encoding [S: OI 18.10.1.22 f]:** `baseItem` uses the magic values **1048828 for (previous)** and **1048829 for (next)**. The default `1048832` means no base item.
- **Base field rule [D: SUMFN]:** "The base field shouldn't be the same field that you chose".
- **Web:** Show Values As works on the web via right-click [D: CREATE].
- **Graph:** no API.
- **Values effect:** every Show Values As mode is **V**.
- **Office.js:** `DataPivotHierarchy.showAs = {calculation, baseField, baseItem}`, ExcelApi 1.8. Source: https://learn.microsoft.com/en-us/javascript/api/excel/excel.showascalculation

### 1.5 Totals

| Feature | Description | OOXML (defaults) | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Row grand totals (Grand Total **column**) | — | PTD `rowGrandTotals=true` ("Show grand totals for rows" = the Grand Total column) | V/L | yes | `layout.showRowGrandTotals` | no | OPTS; TOTALS |
| Column grand totals (Grand Total **row**) | — | PTD `colGrandTotals=true` | V/L | yes | `layout.showColumnGrandTotals` | no | OPTS |
| Automatic subtotals | Outer-field subtotal using the data field's own function | `pf defaultSubtotal=true` + `<item t="default"/>` | V/L | yes | `PivotField.subtotals = {automatic:true}` | no | TOTALS; OI 18.10.1.69 ss |
| Custom subtotals (one or many functions) | e.g. Sum and Average subtotal lines for the same field. Also subtotals for an **inner** field | `pf sumSubtotal countASubtotal avgSubtotal maxSubtotal minSubtotal productSubtotal countSubtotal stdDevSubtotal stdDevPSubtotal varSubtotal varPSubtotal` (all default false) + one `<item t="sum|countA|avg|max|min|product|count|stdDev|stdDevP|var|varP"/>` each; rendered `<i t="sum" …>` | V | partial (on/off + position) | `Subtotals {sum,count,average,…}` | no | TOTALS; OI 18.10.1.69 hh–rr |
| No subtotals | — | `defaultSubtotal=false` and no t≠data items | L | yes | `{automatic:false}` | no | TOTALS |
| Subtotal position | Top of group (outline/compact) or bottom | `pf subtotalTop=true` | L | yes (Don't show / On top / On bottom) | `layout.subtotalLocation` AtTop/AtBottom/Off | no | TOTALS; LAYOUT |
| Restriction | A field containing a calculated item can't change its subtotal function | — | V | — | — | — | TOTALS |
| Subtotal hidden page items | Include page-filtered items in subtotals (OLAP MDX subselect only) | PTD `subtotalHiddenItems=false` (Office: page-area hidden items, non-OLAP only) | V | — | no | no | OPTS; OI 18.10.1.73 p, gg |
| Mark totals with * | OLAP sources without subselect | PTD `asteriskTotals=false` (**Office inverts: 1 means asterisks NOT shown**) | F | — | no | no | OPTS; OI 18.10.1.73 d |
| Visual totals (OLAP) | Totals over visible members only | PTD `visualTotals=true`, `x14 visualTotalsForSets=false` | V (OLAP) | — | no | no | [S]; MS-XLSX |
| Allow multiple filters per field | Label/value filters + manual filter together (non-OLAP) | PTD `multipleFieldFilters=true` | V | yes ("Layout" tab) | `PivotTable.allowMultipleFiltersPerField` (1.12) | no | OPTS; FILTER |

### 1.6 Sorting

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Sort by label A→Z / Z→A | Items in ascending/descending caption order (locale-dependent; not case-sensitive) | `pf sortType="manual"` (default) / `ascending` / `descending`; item order = order of `<item>` | L | yes | `PivotField.sortByLabels(sortBy)` | no | SORT |
| Sort by values | Order items by a data field's value in a given column/row scope | `sortType` + `<autoSortScope><pivotArea><references><reference field x/>…` | L | yes (Sort by value) | `sortByValues(sortBy, dataHierarchy, scope)` (1.9), `layout.setAutoSortOnCell(cell, dir)` | no | SORT; JS |
| Manual sort (drag/Move) | Arbitrary item order | `sortType="manual"` and the `<items>` order | L | yes (drag, right-click Move) | no | no | SORT |
| Sort automatically on update | AutoSort re-applies on refresh | `sortType≠manual`; `pf nonAutoSortDefault=false` (Office: true = simple data sort) | L | ? | no | no | SORT; OI 18.10.1.69 g |
| Custom lists | Month/day lists and user lists drive order. Lost on refresh per SORT | PTD `customListSort=true` ("Use Custom Lists when sorting") | L | ? | `PivotTable.useCustomSortLists` (1.9) | no | SORT; OPTS |
| Data source order | OLAP only | `pf dataSourceSort` (must not be 1 for non-OLAP) | L | — | no | no | SORT; OI 18.10.1.69 x |
| Field-list sort | A→Z vs data-source order in the field list | PTD `fieldListSortAscending=false` | M | — | no | no | OPTS |

### 1.7 Filtering

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Manual item filter (checkbox list) | Hide items | `item h="1"` (hidden) on row/col/page items; Excel also writes `s`? (Office: `s` means "manual filter applied") | V (rows vanish; totals change) | yes | `PivotManualFilter {selectedItems}`; `PivotItem.visible` | no | FILTER; OI 18.10.1.45 a |
| Keep only / hide selected | Same as manual filter | as above | V | yes | — | no | FILTER |
| Include new items in manual filter | Whether new items after refresh are visible | `pf includeNewItemsInFilter=false` (ignored for OLAP); `hideNewItems` OLAP only | V | ? | no | no | TOTALS; OI 18.10.1.69 s, uu |
| Report (page) filter | Field in Filters area. Single item or "(All)" | `pf axis="axisPage"`; `<pageField fld item>` (no `item` = all); `hier=-1` for non-OLAP | V | yes | `filterHierarchies.add`; `FilterPivotHierarchy.enableMultipleFilterItems` | no | FILTER; OI 18.10.1.62 |
| Multiple items in report filter | "Select Multiple Items" | `pf multipleItemSelectionAllowed=true` + `item h=1` on unselected; `pageField@item` must then be absent | V | yes | `enableMultipleFilterItems` | no | OI 18.10.1.62 a, g, i |
| Label filters | Equals / Does not equal / Begins with / Does not begin with / Ends with / Does not end with / Contains / Does not contain / >, ≥, <, ≤ / Between / Not between (on item captions) | `<filters><filter fld! type! id! evalOrder iMeasureHier iMeasureFld mpFld name description stringValue1 stringValue2><autoFilter ref><filterColumn colId><customFilters>…` with `type` ∈ caption* (captionEqual … captionNotBetween). Office: caption* apply to item captions | V | yes | `PivotLabelFilter {condition, comparator, substring, lowerBound, upperBound, exclusive}` (conditions: beginsWith, contains, endsWith, equals, >, ≥, <, ≤, between, + `exclusive` to invert) | no | FILTER; [S]; OI 18.18.59 |
| Value filters | Item kept if its aggregate (a chosen data field) meets condition | `type` ∈ valueEqual, valueNotEqual, valueGreaterThan(OrEqual), valueLessThan(OrEqual), valueBetween, valueNotBetween; `iMeasureFld` = data field index (default 0) | V | yes | `PivotValueFilter {condition, comparator, lowerBound, upperBound, value, exclusive}` | no | FILTER; JS |
| Top 10 (Top/Bottom N by Items / Percent / Sum) | Keep top or bottom N items, N%, or items whose cumulative sum reaches N | `filter type="count|percent|sum"` + `<autoFilter><filterColumn><top10 top val percent filterVal/>`; legacy AutoShow: `pf autoShow=false topAutoShow=true itemPageCount=10 rankBy` | V | yes (Top 10/Bottom 10) | `PivotValueFilter` condition TopN/BottomN + `selectionType` Items/Percent/Sum + `threshold` | no | FILTER; JS PivotValueFilter; OI 18.10.1.69 r, aa |
| Date filters | Equals / Before / After / Between / Tomorrow / Today / Yesterday / Next/This/Last Week / Month / Quarter / Year / Year to Date / Q1–Q4 / Jan–Dec (`M1`–`M12`) | `type` ∈ dateEqual, dateNotEqual, dateOlderThan(OrEqual), dateNewerThan(OrEqual), dateBetween, dateNotBetween, tomorrow, today, …, yearToDate, Q1..Q4, M1..M12; `x15:pivotFilter useWholeDay!` | V (**volatile**: relative-date filters depend on TODAY at refresh time [K]) | ? | `PivotDateFilter {condition, comparator:{date, specificity}, lowerBound, upperBound, wholeDays, exclusive}`; 38 DateFilterCondition values incl. allDatesInPeriodJanuary…Quarter4 | no | JS DateFilterCondition; [S]; MS-XLSX x15 pivotFilter |
| Filter evaluation order | Order in which several filters on a field apply | `filter evalOrder` (Office: -1 = order irrelevant) | V | — | — | — | OI 18.10.1.33 g |
| Show Report Filter Pages | Copies the pivot onto one new sheet per page-field item | (creates new sheets + pivots) | V (new sheets) | no [K] | no | no | FILTER |
| Search box in filter | UI-only | — | M | yes | — | — | FILTER |
| Filter items displayed | Filter drop-down shows ≤10,000 items | — | M | — | — | — | LIMITS |

### 1.8 Grouping

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Numeric range grouping | Bins "start-end" with Starting at / Ending at / By | `cf/fieldGroup base` + `<rangePr autoStart=true autoEnd=true groupBy="range" startNum endNum groupInterval=1>` + `<groupItems>` (labels as `s`, plus `<start` / `>end` items) | V (new items; aggregates per bin) | no (community) | no | no | GROUP; [S]; OI 18.10.1.78 |
| Date grouping | Seconds / Minutes / Hours / Days (N days) / Months / Quarters / Years. Several levels create extra cache fields (e.g. Years, Quarters) | `rangePr groupBy="seconds|minutes|hours|days|months|quarters|years" startDate endDate groupInterval`; multi-level: extra cacheFields with `databaseField="0"` and `fieldGroup par=… base=…` (parent `groupBy` must be > child's) | V | no (community: "Excel for the Web doesn't support grouping") | no | no | GROUP; OI 18.10.1.30, 18.10.1.78 a,b,g |
| Auto date/time grouping | Adding a date field to rows/cols automatically groups it (e.g. months, quarters, years) | as above (written into the file) | V | ? | ? (adding via JS may trigger it [K]) | no | GROUP ("Excel automatically detects relationships among them and groups them together"); disable via File > Options > Data > "Disable automatic grouping of Date/Time columns in PivotTables" [K, third-party] |
| Manual (selection) grouping | Group arbitrary items. Creates a new parent field (e.g. "Region2") with items "Group1", … | new cacheField `databaseField=0`, `fieldGroup base` + `<discretePr count><x v/>…` (maps each base shared item → groupItems index) + `<groupItems>` | V | no | no | no | GROUP; OI 18.10.1.26 |
| Ungroup | Removes fieldGroup | — | V | no | no | no | GROUP |
| Restrictions | rangePr and discretePr are mutually exclusive. `groupItems` is required. The `base` field must be a database field. Range grouping needs no strings and no mixed types; a field with mixed types forces `groupBy=range`. Calculated items require ungrouping first | — | V | — | — | — | OI 18.10.1.30/.78 l,m; CALC |

### 1.9 Calculated fields & calculated items

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Calculated field | New field = formula over **sums** of other fields (e.g. `=Sales*1.2`) | PCD `<cacheField name formula="Sales*1.2" databaseField="0">` (Pivot formula); PTD `pf dataField=1`, `df` on it | V | renders/recalcs; **cannot create** (Q&A) | no | no | CALC; OI 18.10.1.3 g; Q&A link in section 0 |
| Calculated item | New item inside a field, formula over items of that field (record-level) e.g. `=Dairy*115%`, `Region[North]`, `Region[-1]` | PCD `<calculatedItems><calculatedItem field formula><pivotArea>…` (`field` = cacheField index per Office); the item also appears in `pf/items` with `f="1"`; per-cell overrides as further calculatedItem entries | V | ? (render yes [K]) | no | no | CALC; OI 18.10.1.8; OI 18.10.1.45 f |
| Solve order | Order of evaluation when calculated items intersect | order of `<calculatedItem>` elements [K] | V | — | — | — | CALC ("Solve Order") |
| List formulas | Writes a sheet listing formulas | UI | M | ? | no | no | CALC |
| Formula grammar limits | No cell refs, names, structured refs, array constants, UDFs. No GETPIVOTDATA, D-functions, INDEX, RAND, NOW, TODAY, AREAS, ROWS, COLUMNS, OFFSET, CELL, INDIRECT, INFO, RANDBETWEEN. Can't refer to totals | — | V | — | — | — | MS-OI29500 Pivot Formulas; CALC |
| OLAP calculated members/measures | MDX members/measures; DAX measures for the Data Model | `calculatedMembers`, `x14:calculatedMember` | V | partial | no | no | CALCITEMS-SVC; out of scope |

### 1.10 Formatting / styles / number formats

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| PivotTable style | Built-in/custom style + style options | PTD `<pivotTableStyleInfo name showRowHeaders showColHeaders showRowStripes showColStripes showLastColumn>` (Excel defaults 0 for the show* attributes; `showLastColumn` ignored); custom styles in styles.xml `<tableStyles>` (`pivot="1"`) with pivot-specific `tableStyleElement` types (pageFieldLabels, firstSubtotalRow, blankRow, …) [K] | F | yes (gallery) | `workbook.pivotTableStyles` (PivotTableStyle: name, readOnly, delete, duplicate); no per-pivot setter in fetched API pages | no | LAYOUT; OI 18.10.1.74 |
| Legacy AutoFormat | Pre-2007 formats | PTD `useAutoFormatting=false`, `autoFormatId` (Office default 1; valid <21 or 4097..4117), `apply*Formats` (Office default 0) | F | — | `layout.autoFormat` (1.9) | no | OI 18.10.1.73 b,s,t–y |
| Preserve cell formatting on update | Keep user formats across refresh/pivoting | PTD `preserveFormatting=true` (Office: 1 = preserve) | F | ? | `layout.preserveFormatting` (1.9) | no | OPTS; OI 18.10.1.73 o |
| Autofit column widths on update | Resize columns on refresh | PTD `applyWidthHeightFormats` [K mapping]; web "Autofit column widths on refresh" | F | yes | `layout.autoFormat` covers row/col sizing | no | OPTS; REFRESH |
| Pivot-area formats | Formats bound to pivot areas (survive refresh) | `<formats><format action="formatting|blank" dxfId><pivotArea type dataOnly labelOnly grandRow grandCol cacheIndex outline offset collapsedLevelsAreSubtotals axis fieldPosition><references>…` (Excel doesn't support action drill/formula) | F | ? | no | no | [S]; OI 18.10.1.35 |
| Conditional formatting on pivots | CF scoped to selection / all values of a field / data field | `<conditionalFormats><conditionalFormat scope type priority><pivotAreas>`; matching sheet `<conditionalFormatting pivot="1">` with cfRule of same priority; scope data/selection ⇒ type all/none only | F | ? | (generic CF API) | no | OI 18.10.1.18; x14 conditionalFormats |
| Number formats | see 1.3 | `df numFmtId`, `pf numFmtId`, `cf numFmtId` | F | | | | |
| Server formats (OLAP) | Number/font/fill from server | `serverFormats`, `x15:pivotTableServerFormats` | F | — | — | — | OPTS |

### 1.11 Refresh & lifecycle

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Refresh | Rebuild cache from source and re-render | (rewrites PCD/PCR/PTD) | V | yes | `PivotTable.refresh()`, `PivotTableCollection.refreshAll()` | **yes**: `POST /workbook/worksheets/{id}/pivotTables/{id}/refresh`, `…/pivotTables/refreshAll` | REFRESH; Graph |
| Refresh on open | — | PCD `refreshOnLoad` | V | yes | `refreshOnOpen` | no | REFRESH |
| Auto Refresh (2025) | Pivots on local data refresh when source changes; per data source; on by default for new local-data pivots; not for external/async sources; may be unavailable with volatile functions (RAND/NOW) or old-version co-authoring | not documented | V | yes (settings pane) | ? | no | REFRESH; AUTOREFRESH |
| Defer layout update | Field-list option; postpone re-render until Update | none in file (UI; VBA `ManualUpdate`) [K] | — | ? | (batching via `context.sync`) | no | [K] |
| Change data source | Re-point cache at a new range | PCD worksheetSource | V | ? | no (no setter found) | no | REFRESH mentions button |
| Enable Show Details (drill-down) | Allow double-click → new sheet with source records | PTD `enableDrill=true` | V (creates sheet) | ? | no | no | OPTS; DETAILS |
| Enable wizard / field list / properties | UI toggles | PTD `enableWizard=true`, `disableFieldList=false`, `enableFieldProperties=true` | M | — | `layout.enableFieldList` (1.10) | no | [S] |
| Edit values area (what-if) | OLAP write-back | PTD `editData=false`; `x14 enableEdit autoApply allocationMethod weightExpression`; `pivotEdits/pivotChanges` | V (OLAP) | — | `PivotTable.enableDataValueEditing` (1.9) | no | OPTS; MS-XLSX |
| Delete pivot | — | — | V | yes | `PivotTable.delete()` | no | JS |
| Published / immersive / version | Misc flags | PTD `published=false`, `immersive=true` (Office: 1 = on), `createdVersion`, `updatedVersion`, `minRefreshableVersion` | M | — | — | — | OI 18.10.1.73 q,r |

### 1.12 Interaction / drill-down

| Feature | Description | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|
| Show Details (double-click a value) | New worksheet listing the source records behind the cell; a new sheet each time | V | ? | no | no | DETAILS |
| Expand/collapse (+/−), Expand/Collapse entire field, drill down/up (OLAP) | Hides inner items | L | yes | `PivotItem.isExpanded` | no | DETAILS |
| Field list / drag between areas | — | V | yes | hierarchies API | no | WEBDIFF ("Fields List works in a browser…") |
| Contextual tooltips | — | M | ? | no | no | OPTS (`showDataTips=true`, `showMemberPropertyTips`) |
| `getDataHierarchy(cell)`, `getPivotItems(axis, cell)` | Identify which data field and items produced a cell | — | — | ExcelApi 1.9 | no | JS PivotLayout |

### 1.13 GETPIVOTDATA

SPEC-0015 already covers this. The documented rules are summarized here for completeness.

- **Syntax:** `GETPIVOTDATA(data_field, pivot_table, [field1, item1], …)`. 1 to 126 pairs, in any order.
- **data_field:** the on-sheet caption or the source name. "Sum of" and "Count of" prefixes may be omitted.
- **pivot_table:** any cell, range or name inside the pivot. If several pivots are in the range, the **most recently created** one is used.
- **Items:**
  - Dates must be serial numbers or `DATE(...)`.
  - Multiple items go in braces.
  - Items may be cell references.
- **Data sources:** calculated fields, calculated items and custom calculations can be used.
- **Return value:** the single cell's value is returned whether it is text, a number, an error or blank.
- **#REF! cases:**
  - `pivot_table` is not inside a pivot.
  - The fields are not visible.
  - A report filter hides the data.
  - No matching data exists.
- **OLAP item syntax:** `"[Product].[All Products].[Foods]"`.
- **Auto-generation:** typing `=` and clicking a pivot cell generates the formula. This is toggled by "Generate GetPivotData" under PivotTable Analyze > Options. **[K]** The toggle is an application setting, not stored in the workbook.

Source: [D: GPD].

### 1.14 Slicers & timelines

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| Slicer | Button UI acting as a manual filter on one field; can connect to several pivots **sharing the same cache** | sheet `x14:slicerList` → slicer part `x14:slicer name cache caption startItem columnCount showCaption level style lockedPosition rowHeight`; workbook-level `slicerCacheDefinition name! sourceName!` with `<pivotTables><pivotTable tabId name>` and `<data><tabular pivotCacheId! sortOrder=ascending customListSort=true showMissing=true crossFilter=showItemsWithDataAtTop><items><i x s nd>`; PCD `x14:pivotCacheDefinition pivotCacheId slicerData` | V (filters connected pivots) | yes for local pivots; not Data Model / table slicers | `slicers.add(pivotOrName, field, sheet)`, `selectItems`, `clearFilters`, `getSelectedItems`, `slicerItems` (key, name, isSelected, hasData), `sortBy`, `style`, `caption` | no | SLICER; JS overview; MS-XLSX via SDK |
| Slicer cross-filtering display | Items with no data shown at top / greyed / hidden; deleted items shown | `tabular crossFilter`, `showMissing` | M | ? | `SlicerItem.hasData` | no | MS-XLSX |
| Timeline | Date slider filtering by years/quarters/months/days; needs a date field. Coexists with a slicer on the same field only with "Allow multiple filters per field" | `x15:timelineCacheDefinition name sourceName` + `<pivotTables>` + `<state singleRangeFilterState=true minimalRefreshVersion lastRefreshVersion pivotCacheId filterType><selection startDate endDate/><bounds…/><movingPeriodState>`; `x15:timeline name cache caption level selectionLevel scrollPosition showHeader showSelectionLabel showTimeLevel showHorizontalScrollbar style`; the filter itself is a `<filter type="dateBetween">` on the PTD [K] | V | "generally work" (view/use) | no | no | TIMELINE; WEBDIFF; MS-XLSX x15 |

### 1.15 PivotCharts (see section 3 for engine needs)

| Feature | Description | OOXML | Values? | Web | JS | Graph | Sources |
|---|---|---|---|---|---|---|---|
| PivotChart | Chart bound to a PivotTable. Its data is the pivot's data, and filtering either one filters the other | chart part `c:chartSpace/c:pivotSource` (`c:name` = `[Book.xlsx]Sheet!PivotTable1` [K format], `c:fmtId` ≤ 2147483647, ≤1 extLst); `c:chart/c:pivotFmts/c:pivotFmt (idx, spPr, marker, dLbl; Office disallows txPr)`; `c15:pivotSource` (x15 ext copy); PTD `chartFormat` (PivotChart format id) + `<chartFormats><chartFormat chart! format! series=false><pivotArea>` (`format` → `c:pivotFmt` index per Office) | V (chart caches) | create via Insert Chart on a selected pivot [D: PCHART] | no | no (charts API exists, pivot binding no) | OVERVIEW; PCHART; OI 21.2.2.142–144, 18.10.1.12 |
| Field buttons on chart | Filter/category/series/value buttons on the chart | `c14:pivotOptions` with `dropZoneFilter`, `dropZoneCategories`, `dropZoneData`, `dropZoneSeries`, `dropZonesVisible` (all CT_BooleanFalse); only if a `c:pivotSource` exists. **[K]** `c16:pivotOptions16/showExpandCollapseFieldButtons` | F | ? | no | no | MS-XLSX via SDK (office2010.drawing.charts.pivotoptions) |
| Chart types | Any type except XY (scatter), stock and bubble. Mac docs: only column/line/pie/radar; treemap, statistical and combo "do not work with PivotTables". Forum reports treemap and waterfall disallowed on Windows | `c:plotArea` | — | ? | — | — | OVERVIEW; PCHART |
| No re-orientation via Select Data | Swap rows/cols by pivoting the pivot | — | — | — | — | — | OVERVIEW |
| Formatting on refresh | Kept, except trendlines, data labels, error bars and changes to specific series | `c:pivotFmts` re-applied by series index | F | — | — | — | OVERVIEW; LAYOUT |
| Chart from chart | A PivotChart cannot be based on another PivotChart | — | — | — | — | — | OVERVIEW |

### 1.16 Data Model / OLAP / external / AI

| Feature | Notes | Sources |
|---|---|---|
| Data Model pivots (Power Pivot): measures, Distinct Count, relationships | **Out-of-scope candidate.** The web can view, refresh and use slicers but not create these. Office.js has no support. | SUMFN; CREATE; JS overview |
| OLAP cube pivots | **Out of scope.** Server-calculated; no calculated fields or items; no custom subtotal function. | CALC "OLAP" |
| Power BI dataset pivots | Desktop; slicers need Windows/Mac | SLICER |
| Recommended PivotTables / "Insert > PivotTable" recommendations | Generates ordinary pivots. AI-based on the web and M365-only. Not a file-format feature. | CREATE |
| Copilot / Analyze Data | Produces normal pivots. Not a file feature. | [K] |
| PIVOTBY / GROUPBY | Worksheet functions, not pivot tables (SPEC-0006). Behavior may differ from pivots. | SPEC-0015 |

---

## 2. Documented quirks and rules (each with source)

### Aggregation and cell values

1. **Count works like COUNTA.** Count Numbers works like COUNT. [SUMFN]
2. **Default summary function.**
   - Sum for numeric value fields. Count when the field is non-numeric or has blanks.
   - Microsoft wording: "Count is the default for non-numeric or blank value fields" [CALC web section].
   - Web wording: "If Excel reads the data as text, it uses COUNT" [CREATE].
   - **[K] Probe:** does one blank cell in a numeric column flip the default to Count?
3. **Every cell aggregates the source records.**
   - "Source data is summarized (not the displayed value)" [SUMFN].
   - So a subtotal or grand total of Average, StdDev, etc. is computed from the records, not from the subtotals. **[K]** This is the standard pivot semantics; SUMFN's wording implies it. Probe Average and Var grand totals.
4. **Calculated fields operate on the SUM of the underlying data.**
   - `=Sales*1.2` multiplies the summed Sales for each cell, not each record. This holds at subtotal and grand-total level too. A product like `Price*Qty` therefore gives sum(Price)×sum(Qty) in totals. [CALC]
   - **[K] Probe:** what happens when the data field over a calculated field is set to Count or Average? SUMFN says the summary function "can't be changed" for calculated fields.
5. **Calculated items operate on individual records.**
   - `=Dairy*115%` multiplies each Dairy record before summing. [CALC]
   - A field containing a calculated item cannot change its subtotal function. [TOTALS]
   - The summary function can't be changed for fields with calculated items. [SUMFN]
   - Calculated items require the field to be ungrouped first. [CALC]
6. **Calculated item references** [CALC]:
   - Items can be qualified as `Region[North]` to avoid #NAME? when two fields share an item name.
   - Names with spaces, numbers or symbols need single quotes.
   - Items can be referenced **by position in the sorted, displayed report**. Hidden items are not counted.
   - `Region[-1]` and `Region[+1]` are relative to the calculated item. Positions out of range give **#REF!**.
7. **Solve order:** when calculated items or formulas intersect, the "Solve Order" list decides precedence. [CALC]
8. **Pivot formulas cannot use** [MS-OI29500 Pivot Formulas; CALC]:
   - cell references, defined names, structured references or array constants;
   - user-defined functions;
   - GETPIVOTDATA, D-functions, INDEX, RAND, NOW, TODAY, AREAS, ROWS, COLUMNS, OFFSET, CELL, INDIRECT, INFO or RANDBETWEEN;
   - totals (for example "March Total").

### Show Values As

9. **Index** = (cell × grand total of grand totals) / (grand row total × grand column total). [SVA; JS ShowAsCalculation]
10. **% of Parent Row/Column Total** = value / parent item's value on that axis. **% of Parent Total** = value / value of the parent item of the selected base field. [SVA]
11. **Difference From with a missing base item shows `#N/A`.**
    - Office.js docs: "If 'A Farms' did not sell a particular type of fruit, '#N/A' is displayed" (image caption in the differenceFrom example). [JS overview]
    - **[K]** % Of and % Difference From probably behave the same way. Probe it.
12. **Base item `(previous)` and `(next)`** are stored as 1048828 and 1048829. They are only allowed for difference, percent and percentDiff. [OI 18.10.1.22 f]
13. **Show-as modes needing a base field** in Office [OI 18.10.1.22 e; MS-XLSX x14 dataField]:
    - difference, percent, percentDiff and runTotal require `baseField` ≥ 0 and < the number of pivotFields;
    - percentOfParent, percentOfRunningTotal and the two rank modes also need `baseField` ≥ 0.
14. **Base field restriction:** "The base field shouldn't be the same field that you chose." [SUMFN]

### Totals and filters

15. **Grand-total naming** [OPTS]:
    - "Show grand totals for rows" shows the Grand Total **column**.
    - "Show grand totals for columns" shows the Grand Total **row**.
16. **Filtered items in totals** [OPTS; TOTALS]:
    - "Subtotal filtered page items" and "Mark totals with *" are **OLAP-only**.
    - For non-OLAP pivots, totals cover visible items only. **[K]** Probe manual filters vs value/Top 10 filters vs page filters.
    - `subtotalHiddenItems` controls whether hidden **page-area** items count (non-OLAP only). [OI 18.10.1.73 p, gg]
17. **Inverted `asteriskTotals`:** Office treats `asteriskTotals=1` as **asterisks not displayed**. [OI 18.10.1.73 d]
18. **Show items with no data** [OPTS; LAYOUT; JS `showAllItems`]:
    - The table-level options for rows and columns are OLAP-only (`showEmptyRow`, `showEmptyCol`).
    - Non-OLAP uses the per-field setting (`pivotField@showAll`).
19. **Page field item rules** [OI 18.10.1.62]:
    - With no `item`, all items are shown (when not multi-select).
    - With multi-select, `item` must be absent and the hidden items carry `h=1`.
    - `hier=-1` for non-OLAP.
20. **Value-filter targeting:** value filters name the data field through `iMeasureFld` (default 0). caption* filter types apply to **item** captions. [OI 18.10.1.33; OI 18.18.59]
21. **Allow multiple filters per field** lets a slicer and a manual filter, or a label and a value filter, coexist. Non-OLAP only. Timelines need it to coexist with a slicer on the same field. [OPTS; TIMELINE]

### Sorting and grouping

22. **Sort rules** [SORT]:
    - Text sorts alphabetically and **case-insensitively** ("You can't sort case-sensitive text entries").
    - Leading spaces affect the order.
    - The order is locale-dependent.
    - Custom-list order is "not retained when you update (refresh)".
    - "First key sort order" (a custom list) is available only when AutoSort is off.
    - **[K]** New items after a refresh are appended at the end of a manually sorted field. Probe it.
23. **Group naming and labels** [GROUP]:
    - Grouping adds new group fields.
    - The UI names: "Starting at", "Ending at", "By", "Number of days".
    - Auto time grouping happens "when you add rows of time-related fields".
    - **[K]** Common labels, to probe:
      - month groups: "Jan".."Dec";
      - quarter groups: "Qtr1".."Qtr4";
      - year groups: "2024";
      - day groups: "1-Jan";
      - date bounds: "<1/1/2024" and ">12/31/2024";
      - numeric bins: "0-9", "10-19";
      - manual groups: "Group1", and a new field named "<Field>2";
      - auto date grouping adds "Years" and "Quarters" fields, and the date field shows months;
      - blank items display as "(blank)".
24. **Grouping structure** [OI 18.10.1.30, 18.10.1.78]:
    - A date-grouped hierarchy's parent fields have strictly coarser `groupBy` values.
    - The `groupInterval` for date parts is an integer below 32768.
    - Missing `startNum`/`endNum`/`startDate`/`endDate` are computed from the shared items.

### File-format notes

25. **Items and cache rules** [OI 18.10.1.45, 18.10.1.90]:
    - Item `x` indexes **sharedItems**.
    - `t="default"` (and subtotal) items must come after the data items. **[K]** Excel always writes them last.
    - The values `blank` and `grand` are not allowed in `<item t>`, only in `<i t>`.
    - Booleans and errors count as "strings" for `containsString` and `containsMixedTypes`.
    - `longText` is false when text is ≤255 characters.
26. **Limits** [LIMITS]:
    - 1,048,576 unique items per field.
    - 256 value fields.
    - 256 report filters.
    - 10,000 items in filter drop-downs.
    - 32,767-character strings.
    - Row and column fields are limited only by memory.
    - [OI] ≤16,384 cache fields; 1,048,576 `<i>` per axis; ≤256 page fields.
27. **Cache sharing:** refreshing, grouping or ungrouping, and adding calculated fields or items in one pivot affects every pivot on the same cache. [OVERVIEW]
28. **Value-field duplicates:** adding the same field again creates "_2", "_3" and so on. [SVA]
29. **GETPIVOTDATA:** when a range spans several pivots, the most recently created one wins. A report filter that hides the data gives #REF!. [GPD]
30. **Pivot #SPILL! (Insider):** a pivot that can't expand collapses to a single top-left `#SPILL!`. Dependents (GETPIVOTDATA, PivotCharts, macros) are affected; slicers and timelines are not. [PSPILL]
31. **Auto Refresh (Insider)** [REFRESH; AUTOREFRESH]:
    - It is per data source and on by default for new pivots on local workbook data.
    - Changes to external sources are not seen. Asynchronous sources are not supported.
    - It may be unavailable with volatile functions or in older-version co-authoring.
32. **Preserve formatting** keeps formats, but PivotChart trendlines, data labels, error bars and series-specific changes are lost on refresh. [LAYOUT; OVERVIEW]
33. **Other spec inversions and clarifications** [OI 18.10.1.73, 18.10.1.45, 18.10.1.69]:
    - `preserveFormatting=1` means preserve.
    - `immersive=1` means on.
    - `indent` 127 means none.
    - `item@sd` means expanded.
    - `item@s` means a manual filter is applied.
    - `pivotField@nonAutoSortDefault=1` means a simple data sort.
    - In `subtotalCaption`, a `?` is replaced by the field name.
34. **Required item rows:** for each `*Subtotal=true` attribute, the field's `<items>` must contain the matching `<item t=…>`. `defaultSubtotal` (true or absent) requires `<item t="default"/>`. [OI 18.10.1.69 hh–ss]
35. **PivotChart limits:** a PivotChart can't be built on another PivotChart. The Select Data dialog can't change the chart's range or orientation. [OVERVIEW]
36. **Excel for the web gaps** (community and Q&A sources, not official):
    - Can't create calculated fields.
    - Can't group.
    - Only local-pivot slicers can be created.
    - Can view and edit OLAP pivots but not create them.
    - Recommended PivotTables require a Microsoft 365 subscription.

    Sources: https://learn.microsoft.com/en-us/answers/questions/5918589/excel-web-pivot-table-calculations; https://techcommunity.microsoft.com/discussions/excelgeneral/group-in-pivot-table/4054086; [SLICER]; [CALC]; [CREATE].

---

## 3. What PivotCharts need from an engine beyond pivot tables

These points come from OVERVIEW, PCHART, `dml-chart.xsd`, and the MS-OI29500 notes 21.2.2.142–144 and 18.10.1.12. Items tagged **[K]** need probing.

1. **The chart's data is the pivot's rendered body** [D: OVERVIEW, "PivotCharts are based on their associated PivotTable's data source"; Select Data can't change the range].
   - The engine must produce, per chart, the visible row items as the category axis and the visible column items × data fields as the series.
   - **[K]** Grand totals and subtotals are **excluded**. Collapsed outer items appear as single aggregated categories. Hidden and filtered items are excluded.
2. **Multi-level category axis.**
   - Nested row fields become multi-level labels. **[K]** In the file these are `c:cat/c:multiLvlStrRef` with a `c:multiLvlStrCache` (one `c:lvl` per row field, innermost first).
   - Series names come from the column-field item path, or the data-field caption when there are no column fields.
3. **Chart caches.**
   - Each series' `c:val/c:numRef` has an `c:f` pointing at pivot body cells and a `c:numCache` holding the values and the number format `formatCode` (taken from `dataField@numFmtId`).
   - Categories use `c:strRef/c:strCache` or `multiLvl…`.
   - The engine must regenerate these caches after any pivot recompute: refresh, filter, slicer, expand or collapse.
   - **[K]** Writers rely on Excel to rebuild the caches. Readers such as our importer would see stale caches unless we recompute.
4. **Pivot binding.**
   - `c:chartSpace/c:pivotSource` holds `c:name` (the pivot reference) and `c:fmtId`.
   - The PTD's `chartFormat` attribute and `<chartFormats><chartFormat chart format series><pivotArea>` tie per-series and per-point formats to pivot areas. Office says `format` indexes `c:pivotFmt`.
   - So formats must be re-attached to series **by pivot identity, not by position**, when the layout changes.
   - Excel disallows `txPr` in `pivotFmt`.
5. **Field buttons.** `c14:pivotOptions` (`dropZoneFilter`, `dropZoneCategories`, `dropZoneData`, `dropZoneSeries`, `dropZonesVisible`) is visual only. It exists only when `c:pivotSource` exists [MS-XLSX].
6. **Filtering through the chart** filters the pivot and vice versa [D: OVERVIEW "changes … are reflected in each other"; PCHART (Mac): filtering the pivot or slicers filters the chart].
   - The engine model therefore needs a single filter state shared by a pivot, its slicers and timelines, and its charts.
7. **Chart-type validation.**
   - Reject or ignore XY scatter, stock and bubble [OVERVIEW].
   - Treemap, sunburst, histogram, box & whisker, waterfall, funnel and map are likely unsupported; Mac docs explicitly exclude treemap and statistical charts [PCHART]. **[K]** Windows behavior for the cx: chart types needs a probe.
8. **Refresh semantics.** Formatting is kept on refresh except trendlines, data labels, error bars and series-specific changes [OVERVIEW]. Series count can change on refresh, so the series list is derived, not stored.
9. **Limits.** 256 value fields and 256 report filters per PivotChart [LIMITS]. 255 data series per chart [LIMITS, charting].
10. **Interaction with #SPILL!.** A blocked pivot affects its PivotCharts [PSPILL]. **[K]** Probably the chart goes empty.
11. **Oracle reach.**
    - Graph has a charts API, but no PivotChart binding.
    - Office.js has no PivotChart API in the fetched pages.
    - The only oracle path is an authored xlsx: upload it, refresh, then read the chart through the Graph chart series/points endpoints. **[K]** Whether Graph reports pivot-chart series values has not been verified.

---

## 4. API reach (Office.js vs Graph), precisely

### Microsoft Graph (v1.0 and beta are identical)

- **Resource:** `workbookPivotTable { id, name }` with relationship `worksheet`.
- **Methods:**
  - `GET /workbook/worksheets/{id}/pivotTables` (list)
  - `GET …/pivotTables/{id}` (get)
  - `POST …/pivotTables/{id}/refresh`
  - `POST …/pivotTables/refreshAll` (collection-only)
- **Nothing else.** There is no create, layout, field, filter or data-field access.
- **How the oracle uses it:**
  - Rendered output is read with the generic range APIs, for example `worksheets/{id}/range(address=…)` or `usedRange`.
  - GETPIVOTDATA is evaluated by writing formulas into cells.

Sources: https://learn.microsoft.com/en-us/graph/api/resources/workbookpivottable?view=graph-rest-1.0 and the `?view=graph-rest-beta` version of the same page.

### Office.js (ExcelApi), reachable only from add-ins and Office Scripts, not from Graph

| Object | Members | API set |
|---|---|---|
| `PivotTableCollection` | `add(name, source: Range|string|Table, destination)`, `getItem`, `getCount`, `refreshAll` | |
| `PivotTable` | `name`, `id`, `worksheet`, `rowHierarchies`, `columnHierarchies`, `dataHierarchies`, `filterHierarchies`, `hierarchies`, `layout` | 1.3–1.8 |
| `PivotTable` | `allowMultipleFiltersPerField` | 1.12 |
| `PivotTable` | `enableDataValueEditing` | 1.9 |
| `PivotTable` | `refreshOnOpen` | 1.13 |
| `PivotTable` | `useCustomSortLists` | 1.9 |
| `PivotTable` | `delete()`, `refresh()` | |
| `PivotTable` | `getDataSourceString()`, `getDataSourceType()` | 1.15 |
| `PivotLayout` | `layoutType` (Compact, Outline, Tabular), `subtotalLocation` (AtTop, AtBottom, Off), `showRowGrandTotals`, `showColumnGrandTotals` | |
| `PivotLayout` | `showFieldHeaders`, `fillEmptyCells`, `emptyCellText`, `altTextTitle`, `altTextDescription`, `repeatAllItemLabels()`, `displayBlankLineAfterEachItem()` | 1.13 |
| `PivotLayout` | `autoFormat`, `preserveFormatting` | |
| `PivotLayout` | `enableFieldList` | 1.10 |
| `PivotLayout` | `getRange`, `getDataBodyRange`, `getRowLabelRange`, `getColumnLabelRange`, `getFilterAxisRange` | |
| `PivotLayout` | `getDataHierarchy(cell)`, `getPivotItems(axis, cell)`, `setAutoSortOnCell(cell, dir)` | 1.9 |
| `RowColumnPivotHierarchy` | `name`, `position`, `fields`, `setToDefault()` | |
| `FilterPivotHierarchy` | also `enableMultipleFilterItems` | |
| `DataPivotHierarchy` | `name`, `numberFormat`, `position`, `showAs {calculation, baseField, baseItem}`, `summarizeBy`, `field`, `setToDefault()` | |
| `PivotField` | `name`, `items`, `showAllItems`, `subtotals` (automatic or any of 11 functions), `sortByLabels`, `sortByValues(dir, dataHierarchy, scope)` | |
| `PivotField` | `applyFilter({labelFilter, valueFilter, dateFilter, manualFilter})`, `clearFilter(type)`, `clearAllFilters()`, `getFilters()`, `isFiltered(type)` | 1.12 |
| `PivotItem` | `name`, `isExpanded`, `visible` | |
| `Slicer` | `add`, `selectItems`, `clearFilters`, `getSelectedItems`, `slicerItems` (`key`, `name`, `isSelected`, `hasData`), `caption`, `style`, `sortBy`, geometry | |
| `PivotTableStyle` | collection only (`name`, `readOnly`, `duplicate`, `delete`) | |

Enumerations:

| Enum | Values |
|---|---|
| `AggregationFunction` | Automatic, Sum, Count, Average, Max, Min, Product, CountNumbers, StandardDeviation, StandardDeviationP, Variance, VarianceP (no Distinct Count) |
| `ShowAsCalculation` | the 14 modes plus none and unknown |
| Value filter conditions | Equals, GreaterThan, GreaterThanOrEqualTo, LessThan, LessThanOrEqualTo, Between, TopN, BottomN; `selectionType` Items, Percent or Sum; `threshold` |
| Label filter conditions | BeginsWith, Contains, EndsWith, Equals, GreaterThan, GreaterThanOrEqualTo, LessThan, LessThanOrEqualTo, Between (`exclusive` inverts) |
| `DateFilterCondition` | Equals, Before, BeforeOrEqualTo, After, AfterOrEqualTo, Between, Today, Yesterday, Tomorrow, This/Last/Next Week, Month, Quarter and Year, YearToDate, allDatesInPeriod Quarter1–4 and January–December; `comparator {date, specificity}` and `wholeDays` |

**Office.js can't do** any of the following:

- grouping of any kind;
- calculated fields or items;
- Distinct Count;
- Data Model, OLAP or Power Pivot ("PivotTables created with OLAP are not currently supported. There is also no support for Power Pivot.");
- PivotCharts;
- Show Report Filter Pages or Show Details;
- number-of-items-to-retain;
- error-cell text (`showError` / `errorCaption`);
- merge labels, indent, page-field arrangement;
- per-pivot style assignment (none found);
- custom subtotal or grand-total captions;
- changing the source.

Sources: JS overview and the class pages listed under "Primary references".

---

## 5. Excel for the web vs desktop: summary

| Area | Web status | Source |
|---|---|---|
| Display of existing pivots and PivotCharts | yes ("displayed in a browser window similar to how they are displayed in Excel"). Exceptions: charts in chart groups, external references, unsupported sources | WEBDIFF |
| Field list, rearranging fields | yes | WEBDIFF; CREATE |
| Create pivot (Insert > PivotTable, recommended pivots) | yes. Recommendations need M365 and AI connected services | CREATE |
| Summarize Values By (11 functions) | yes. Sum is the numeric default, Count otherwise | CALC (web section); CREATE |
| Show Values As | yes (right-click) | CREATE |
| Sort (asc/desc, by value, manual drag) | yes. More Sort Options and AutoSort are not described | SORT |
| Filters (manual, label, value, Top 10/Bottom 10), report filters | yes | FILTER |
| Settings pane: grand totals, subtotals (none/top/bottom), repeat labels, blank rows, empty/error cell text, autofit on refresh, refresh on open, Auto Refresh, allow multiple filters, field captions | yes | LAYOUT; REFRESH; FILTER |
| Refresh / Refresh All | yes | REFRESH |
| Slicers | create for local pivots only; use all | SLICER; FILTER |
| Timelines | "generally work". Creation on the web is undocumented | WEBDIFF |
| PivotChart creation | yes (Insert Chart while a pivot is selected) | PCHART |
| Grouping (numeric, date, manual) | **no** (community) | techcommunity links in section 2, item 36 |
| Calculated fields / items | render yes; **create no** (Q&A, June 2026) | Q&A link |
| Data Model / Distinct Count / OLAP create | no | CREATE; CALC; SLICER |
| Show Report Filter Pages, Show Details | ? (undocumented) | — |
| PivotTable #SPILL! | not announced for web | PSPILL |

---

## 6. Gaps worth an excel-probe early

These are the questions whose answer changes the engine design.

1. Does a Graph `refresh` of an authored workbook recompute everything listed below in Excel for the web? This decides whether the web oracle can evidence those features at all.
   - `fieldGroup` (date, numeric and manual groups)
   - `calculatedItem` and calculated fields
   - `x14` show-as modes
   - filters: Top 10, date and label
   - `showAll`
   - `showError` and `errorCaption`
2. What is the rendered text of item labels?
   - blank → "(blank)"
   - errors
   - booleans
   - group labels and "<"/">" bounds
   - the "Row Labels", "Grand Total" and "Total" captions for each layout form
3. What do Show Values As produce on edge cases? Cover a missing base item (#N/A? blank?), division by zero, (previous)/(next) at the edges, ties in rank, and % of parent at the top level.
4. How are totals affected by filters? Compare manual filter, value filter, Top N and page filter: do totals include hidden items?
5. What is the default summary function for a column with blanks?
6. What is the item order after refresh (new items appended?), and when do custom lists apply by default (`customListSort=1`)?
7. What auto date grouping fields and names does adding a date field via Office.js or the web create, and do they appear in a file saved by the web?

---

## 7. MS-OI29500 notes most relevant to an implementation

All are at https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/ under TOC entries 2.1.737–2.1.803 (§18.10.*), plus 2.1.x for §18.18.59 and §21.2.2.142–144. All 72 notes were fetched as raw text for this research (scratch file `dl/oi_compact.txt`, outside the deliverable).

| Note | Topic | Rule |
|---|---|---|
| 18.10.1.3 | cacheField | name unique and case-insensitive; `formula` = Pivot formula; first field `databaseField=1`; `formula` forbidden for OLAP |
| 18.10.1.8 | calculatedItem | `field` indexes a cacheField; `pivotArea` locates the cells; formula per §18.17 |
| 18.10.1.22 | dataField | `fld` indexes pivotFields; baseField/baseItem rules (1048828 = previous, 1048829 = next) |
| 18.10.1.26 | discretePr | child `x` maps base shared item → groupItems index |
| 18.10.1.29 | field | `x = -2` means data fields; no duplicate `x` |
| 18.10.1.30, 18.10.1.78 | fieldGroup, rangePr | structure constraints (see section 1.8) |
| 18.10.1.33 | filter | `fld` < pivotFields; `iMeasureFld` default 0; `evalOrder -1` |
| 18.10.1.35 | format | `action` drill/formula unsupported |
| 18.10.1.44 | i | `r`/`x`/`v` semantics, 1048832 = no item |
| 18.10.1.45 | item | `x` → sharedItems; `sd` = expanded; `s` = manual filter; `n` unique; f/h/m/sd non-OLAP only; `t` cannot be blank or grand |
| 18.10.1.49 | location | `ref` excludes page fields |
| 18.10.1.62 | pageField | `item` absent = all; multi-select rules; `hier=-1` |
| 18.10.1.67 | pivotCacheDefinition | `r:id` required if saveData and not invalid; `missingItemsLimit` absent = automatic |
| 18.10.1.69 | pivotField | axis absent = no axis; `numFmtId` default 0; subtotal item requirements; `showAll` false for OLAP; `hideNewItems` false for non-OLAP; `itemPageCount` 1..255 |
| 18.10.1.73 | pivotTableDefinition | inversions (asteriskTotals, preserveFormatting, immersive); `indent`; `dataPosition` default 0; showEmptyRow/showEmptyCol OLAP-only; subtotalHiddenItems = page area |
| 18.10.1.74 | pivotTableStyleInfo | show* default 0; `showLastColumn` ignored |
| 18.10.1.77 | r | child count = number of database fields |
| 18.10.1.90 | sharedItems | type-flag semantics; limits |
| 18.10.1.95 | worksheetSource | exactly one of `ref`/`name`; `r:id` = external workbook |
| 21.2.2.142–144 | pivotFmt, pivotFmts, pivotSource | pivotFmt has no txPr; pivotFmts are chart-on-pivot formats; pivotSource `fmtId` ≤ 2^31−1 |
