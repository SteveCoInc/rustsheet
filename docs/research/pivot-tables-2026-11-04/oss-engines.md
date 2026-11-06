# Pivot tables: open-source engines, commercial docs and Excel-authored test corpora

> Research spike of 2026-11-04 (see plan.md in this folder). `<scratch>/` paths were session scratch files and are not committed. Hypotheses here are not Excel evidence until an oracle probe confirms them.

Research date 2026-11-04. All third-party code was read as data only: nothing was built or run, and no
code is copied here. "Claim" means the project says so and RustSheet has not checked it against Excel.
Where an Excel-authored workbook already settles a claim, the evidence is cited.

Clones are in `<scratch>/src/` (shallow, read-only). Helper scripts (ours) are `<scratch>/tools/pivsum.py`
(summarises each pivotTableDefinition: layout form, field counts, features), `tools/app.py`
(Application/AppVersion from docProps, pivot and PivotChart counts) and `tools/cells.py` (dumps a sheet's
cached cells). Per-corpus outputs are `<scratch>/*.piv` and `<scratch>/*.app`.

## 0. Headline findings

1. **mog has the most complete open-source pivot engine (Apache-2.0, about 30k lines in one crate plus about
   10k lines of xlsx I/O).** It reads and writes xlsx, refreshes from the source, renders compact, outline and
   tabular layouts, and supports 11 aggregates, 14 show-values-as modes, filters, top-N, grouping, calculated
   fields and expand/collapse. GETPIVOTDATA reads the rendered cells. None of it is tested against Excel
   output: it has 624 unit tests in compute-pivot, all hand-written expectations.
2. **Calipers has 13 Excel-saved goldens with pivots (19 pivot tables), not 107.** `pv_goldens` is
   "PV" financial models, and only 2 of its 107 have pivots. The 7 `officejs/pivot_*` cases are the most
   useful: an init with no pivot, an Office.js script that builds one, and Excel's rendered output in
   `golden.xlsx`. They are clean, minimal evidence for the default compact layout.
3. **Some of the engines' "Excel" claims are contradicted by Excel-authored files:**
   - mog's number-group labels are `"< 10"`, `"a - b"` and `">= 100"`
     (`compute-pivot/src/grouper.rs:179,189,220`). Excel writes `<10`, `10-14`, `85-90` (the last bin is
     clipped to endNum) and `>90`, and `<10 or (blank)` when the field has blanks. Evidence:
     LO `column-style-autofilter.xlsx` and `pivot-table/shared-numgroup.xlsx` (`<32674`, `32674-47673`).
   - mog's quarter label is `Q1` (`grouper.rs:87`); Excel writes `Qtr1`. mog's Day grouping is the
     day-of-month number (`grouper.rs:110`); Excel's `groupBy="days"` items are 366 `1-Jan`…`31-Dec` labels
     plus `<start` and `>end`. mog also has a "Week N" grouping (`grouper.rs:108`), which Excel does not
     have (weeks are days grouped with `groupInterval="7"`). gridcraft's `Qtr{n}` and `{day}-{Mon}`
     (`engine/src/pivot/items.rs`) do match Excel.
   - Grouped labels follow the locale: `Qrtl1`, `N.év1`, `Mrz` and `янв` all appear in Excel files, and so
     do date-bound labels such as `<5/16/1958`, `<05.01.2015` and `<2020-04-17`. Our en-US target uses
     `Qtr`, `Jan` and `<m/d/yyyy`.
4. **The engines disagree on Rank (show-values-as):**
   - gridcraft: dense ranking (1, 2, 3, 3, 2), claimed as "Excel's" (`engine/src/pivot/compute.rs:360`).
   - mog: competition ranking (1, 1, 3) (`show_values_as/transforms/rank.rs:57-100`).
   - EPPlus: sequential, with no tie handling (`PivotShowAsRankBase.cs`).

   This needs an oracle probe. None of the corpora below has `showDataAs="rankAscending"` or
   `"rankDescending"`.
5. **`rowGrandTotals` and `colGrandTotals`:** gridcraft fixed an inversion (file-format-parity.md change log,
   2026-10-18): `rowGrandTotals` is the Grand Total *row* at the bottom, which the Excel UI calls "On for
   Columns". An Excel-authored file confirms it. In LO `column-style-autofilter.xlsx` pivotTable3,
   `colGrandTotals="0"`, the `<rowItems>` still hold `t="grand"`, and the location `A3:C64` has no Grand
   Total column.
6. **Corpora:** about 150 public Excel-saved workbooks contain about 290 pivot tables, and their rendered
   cells are cached. LibreOffice `sc/qa/unit/data` is the richest source: 64 Excel-saved files, including
   `xlsx/pivot/Pivot*_Row/Column_Grand_Subtotals_Sort*.xlsx`, which target layout variants directly.
   ClosedXML has 38 and POI has 5. RustSheet's existing suites already fetch 22 pivot workbooks:
   Calipers 13, POI 5 and corpus-bench 4.

## 1. mog (fundamental-research-labs/mog)

- **Licence:** Apache-2.0 (`LICENSE`), plus `TRADEMARKS.md`. Commit `c8dd394806b8242f1cad0e3b27f5d58ee7c276cb`.
- **Pivot code** (paths are under `compute/core/` unless noted):
  - `crates/compute-pivot/`: the pure engine, 30.4k lines including tests. Its entry point is
    `(config, data, expansion_state) -> PivotTableResult` (`src/lib.rs`, `src/engine/compute.rs`).
    - `presenter/`: `query_mapping`, `row_flattening`, `column_headers`, `grand_totals`, `visibility`,
      `value_remap`, `result_projection`.
    - `grouper.rs`: date and number grouping.
    - `filter.rs`: include/exclude, conditions, top/bottom.
    - `show_values_as/` (`transforms/{percentage,difference,running,rank,index}.rs`).
    - `calc_field/`: its own lexer, parser and evaluator for calculated fields.
    - `hierarchy/`: tree and flat builders.
    - `engine/drill_down.rs`, `engine/type_detection.rs`, `engine/validation.rs`.
  - `crates/types/pivot-types/` and `/domain-types/src/domain/pivot/`: config, placement, filter,
    show-values-as and expansion types; `ooxml.rs` and `import_identity.rs`.
  - Aggregation, sort and value semantics are delegated to `crates/compute-stats` (`aggregate`, `sort`,
    `values::kahan_sum`). Grouping is delegated to `crates/compute-relational` (GROUP BY and rollup).
  - Rendering to cells: `src/cells/write/pivot_materialization.rs` (394 lines) and
    `src/storage/engine/pivot_materialization.rs` (550 lines).
  - Import: `src/import/parse_output_to_snapshot/pivot_lowering.rs` and `pivot_totals.rs`. The latter
    locates grand-total cells from the OOXML `rowItems`/`colItems` `t="grand"`, "never cell labels".
  - `src/storage/workbook/imported_pivots.rs` and
    `src/storage/infra/hydration/imported_pivot_classification.rs` decide whether an imported pivot is
    "promoted" (becomes editable and recomputed) or kept "unsupported" (preserved only). The reasons are
    `ExternalSource`, `CacheOnlySource`, `FieldCacheMismatch`, `LossyOoxml`, `InvalidOutputRange`, and so on.
  - Export: `src/storage/engine/services/export/pivot_cache_reconciliation.rs` keeps the imported cache
    only while the live pivot still has the same source signature.
  - xlsx I/O, under `/file-io/`:
    - `ooxml/types/src/pivot/*`: typed CT_ structs for cache, table, items, layout and grouping.
    - `xlsx/parser/src/domain/pivot/{parse,reader,write,convert}/*`.
    - `xlsx/parser/src/write/pivot_writer/*`: `config_to_def`, `cache_data`, `cache_sources`.
    - `xlsx/parser/src/output/to_parse_output/pivot_convert/*`: layout, sort, filters, expansion,
      placements.
  - `src/eval/functions/getpivotdata.rs` (225 lines) reads the rendered cells and does not run the pivot
    engine. Wrong argument counts return `#VALUE!` (lines 31-37), unlike the `#REF!` that SPEC-0015
    evidences.
  - Office.js API surface: `/compute/officejs/src/pivot.rs`.
- **Features:**
  - Layouts: `LayoutForm::{Compact, Outline, Tabular}` and `SubtotalLocation::{Top, Bottom}`
    (`domain-types/src/domain/pivot/config.rs:104-120`). Also repeat labels, grand-total toggles and style
    presets.
  - Aggregates: 11, from compute-stats.
  - ShowValuesAs (`domain-types/.../show_values_as.rs:40-66`): NoCalculation, %GrandTotal, %ColumnTotal,
    %RowTotal, %ParentRow, %ParentColumn, Difference, %Difference, RunningTotal, %RunningTotal,
    RankAscending, RankDescending and Index. Its base item can be relative (previous/next).
  - Filters: label and value conditions, include/exclude lists, and top/bottom by items, percent or sum.
  - Grouping: date grouping (year, quarter, month, week, day, hour, minute, second) and numeric intervals.
  - Calculated fields, sort by value, custom sort orders, expand/collapse, drill-down, and field type
    detection.
- **Design choices:**
  - The pivot is compiled into a relational query, executed, then projected back by the presenter.
  - A `ResolvedPivotConfig` is validated once before computing.
  - Show-values-as runs in a three-phase transform trait: leaves, then subtotal peers from deepest depth up,
    then the grand total (`show_values_as/transforms/mod.rs:23-90`).
  - Sums use Kahan summation, which is not Excel-like: Excel's pivot sums are plain double sums. This needs
    a probe.
  - Filter order (`filter.rs:10-16`): include, exclude, condition, showItemsWithNoData, top/bottom. It is
    labelled "per Excel specification", without a source.
  - Rendered output is written as `generated_values` that sit behind authored cells
    (`cells/write/pivot_materialization.rs:1-60`).
- **Excel behaviours it claims** (unverified unless marked):
  - `pivot_materialization.rs:165`: "Compact Excel layout bottom-aligns row-field captions" so that the
    report-filter and "Column Labels" rows sit above them.
  - `pivot_materialization.rs:224`: "Excel leaves repeated outer item labels blank unless repeat labels
    are enabled".
  - The default caption is `"Grand Total"`. The blank item label is `(blank)`
    (`presenter/row_flattening.rs:46`, `column_headers.rs:54`).
  - `filter.rs:158`: above/below-average filters use strict `>` and `<`.
  - `show_values_as/transforms/mod.rs:187-191`: % of grand total applies to subtotals and the grand
    total, which becomes 100%.
  - `calc_field/evaluator.rs:9,47`: field names are case-insensitive.
  - `grouper.rs:8,32-44,123-131`: "Week (Excel convention: Sunday start, Week 1 contains Jan 1)" and "Day
    grouping returns day-of-month… matches Excel". **Both contradict the Excel files in §0.3.**
  - `grouper.rs:214`: "Integer interval: inclusive upper bound (Excel convention)" is consistent with
    `10-14`, but mog's separator and edge labels are not.
- **Tests:** compute-pivot has 624 `#[test]`s, including `tabular_layout_tests/layout_parity.rs`,
  `outline_comparison.rs`, `pivot_rendering_bug_tests.rs`, `engine_property_tests.rs` and the
  value-sorting tie-breakers. A further 165 cover xlsx I/O and materialization. Only one test workbook
  has a pivot: `file-io/xlsx/parser/test-corpus/parity/pivots/pivot-basic.xlsx`. It was built by openpyxl
  plus injected XML (`generators/gen_pivot.py`), so it is not Excel output. Calipers is a submodule at
  `vendor/calipers` (`run-calipers-tests.sh`), so mog's pivot parity against Excel comes only from the
  Calipers goldens below.
- **What to take:**
  - The architecture: a pure engine, a query-shaped presenter, and resolved-config validation.
  - The three-phase show-values-as dispatcher.
  - The import "promote vs preserve" classification with explicit reasons.
  - Locating totals from `rowItems`/`colItems` instead of from labels.
  - Export reconciliation that keeps the original cache while the source is unchanged.
  - Do not take its grouping labels or rank semantics.

## 2. Calipers (fundamental-research-labs/calipers)

- **Licence:** MIT. Commit `9e47ea4fc97b292b15ff7bf3c445acab53f6d8e9`, the same rev that
  `eval/suites/calipers/suite.toml` pins.
- **Provenance:** goldens are produced by Windows desktop Excel 16.0 build 20326, through COM
  (`golden.xlsx.meta.json`). `pv_goldens` (107 cases) was imported from the non-public
  `lyfegame/shortcut` repository, anonymised. "PV" does not mean pivot.
- **Pivot cases:** 13 of 458 goldens; 19 pivot tables; no PivotCharts.

| Case (golden.xlsx) | Pivots | Layout, fields, features |
|---|---|---|
| officejs/pivot_row_data | 1 | compact, 1 row field, 1 data field, A1:B4 |
| officejs/pivot_name | 1 | same as pivot_row_data, named via Office.js |
| officejs/pivot_hierarchy_get_item | 1 | compact, 1 row field, 1 data field |
| officejs/pivot_count | 1 | compact, 1 row field, Count |
| officejs/pivot_filter | 1 | compact, 1 row field, 1 page field (`Product` / `(All)` at A1:B1, body from A3) |
| officejs/pivot_row_col_data | 1 | compact, 1 row and 1 column field: `Sum of Sales` / `Column Labels` / `Row Labels` / `Grand Total` |
| officejs/pivot_two_data | 1 | compact, 2 data fields with no column field: `Row Labels`, `Sum of Sales`, `Sum of Qty`, no "Values" row |
| scratch/pivot_table | 1 | compact, 1 row field, 1 data field |
| roundtrip/pivot | 2 | compact, hidden items, a page field; same content as POI `ExcelPivotTableSample.xlsx` |
| roundtrip/pivot_named_range | 3 | compact with classic drop zones, 2 row fields and 1 column field, page fields, `fieldGroup`, refreshOnLoad, label/value `<filters>`, `showDataAs="index"`, Count, no subtotals |
| roundtrip/formula_stress_test | 1 | compact with classic drop zones, 2 row fields and 2 column fields |
| pv_goldens/pv23_041 | 3 | compact, row and column fields (one pivot has column fields only) |
| pv_goldens/pv23_045 | 2 | tabular, subtotals at the bottom, 1-2 row fields and 1 column field, up to B2:N87 |

- **Note:** the 7 `officejs` cases each have an `init.xlsx` with no pivot and a `script.js`. They are
  ready-made "create pivot → expected render" fixtures for the default compact layout.

## 3. gridcraft (storytold/gridcraft)

- **Licence:** MIT OR Apache-2.0. Commit `632e4a1bc51ee86b3f0a55e95ee78def391aab30`. It describes itself as
  a "clean-room" spreadsheet app, in alpha.
- **Code** (about 7.4k lines):
  - Model: `crates/model/src/features/pivot.rs` (177 lines).
  - Engine: `crates/engine/src/pivot.rs`, plus `pivot/{compute,output,items,charts}.rs`.
  - Calculated fields: `crates/engine/src/pivot_calc.rs`.
  - Commands: `crates/engine/src/cmd/pivot/{create,fields,layout,charts,params}.rs`.
  - xlsx: `crates/xlsx/src/pivot.rs` and `pivot/{read,write}.rs` (write is 907 lines).
  - Tests: `cmd/pivot/tests.rs` (39), `xlsx/src/tests/pivot_tests.rs` (15), `engine/tests/it/pivotchart_link.rs`
    and `pivot_unknown_func.rs`, and UI tests.
- **Features:**
  - Layouts: Compact, Outline and Tabular, with subtotals on or off, top or bottom, and grand totals
    rows/cols.
  - Σ Values can go on rows or columns (`values_on_rows`), at any position.
  - 11 aggregates (`PivotFunc`).
  - Show-as: 5 only (%GrandTotal, %ColumnTotal, %RowTotal, RunningTotal on the first row field, Rank).
  - Date grouping: Years, Quarters, Months and Days. No numeric grouping.
  - Report filters and hidden items.
  - Sort: ascending, descending or source order.
  - Calculated fields, evaluated on sums. Cycles return `#REF!`, with depth and length limits.
  - Show Details (drill-through), PivotCharts with `c:pivotSource` round trip, and styles.
  - Limits: 10M source cells, 2M output cells, 32 axis fields, 256 value fields (`pivot.rs:40-47`).
  - Missing: calculated items, GETPIVOTDATA, slicers, timelines, numeric grouping, %Parent, Difference and
    Index (docs/gaps.md §8).
- **Algorithm:**
  - Items are keyed by a type-tagged identity: numbers by their bits, text lowercased, booleans, `(blank)`.
  - Items sort by class (numbers, then text, then booleans, then blanks), then numerically, then by
    case-folded text (`items.rs:24-60`).
  - Output is a grid of typed `OutCell`s (`CellKind`: Header, Subtotal, GrandTotal…), written as static
    cells.
  - "Lines" are flattened recursively in `output.rs`. Compact and outline put subtotals on the item line;
    tabular puts them below. With Σ Values below a level, there is one subtotal line per value field
    (`output.rs:157-252`).
  - The xlsx writer writes `refreshOnLoad="1"` and one cache per pivot, and leaves `saveData="0"` for big
    sources.
- **Excel behaviours it claims:**
  - Captions:
    - `"Row Labels"` (`compute.rs:199`).
    - `"<item> Total"` for subtotals and `"Total <value name>"` for per-value grand totals
      (`compute.rs:263-270,537`).
    - Values on rows with no column field: the header is `"Total"` (`compute.rs:220`).
    - Count Numbers is captioned `"Count of X"`; the function verbs are `StdDevp` and `Varp`
      (`items.rs:200-215`).
  - Rank is dense (`compute.rs:360`), which **conflicts with mog**.
  - Calculated fields "operate on the sums of the fields" and apply to subtotals and grand totals
    (`pivot_calc.rs:1-8`). This matches Microsoft's documentation.
  - xlsx writer:
    - `rowGrandTotals` is the bottom Grand Total row (`write.rs:669`). This is **corroborated** (§0.5).
    - "Excel counts booleans and errors as strings in the cache field flags" (`write.rs:43`).
    - Two or more value fields with no column field are written with `firstHeaderRow="0"` and x14
      `hideValuesRow="1"`, "so Excel 365 doesn't add a Values row that can turn the report into #SPILL!"
      (`write.rs:699-725`; file-format-parity.md, 2026-10-16). Calipers `officejs/pivot_two_data` shows
      Excel's own render: there is no Values row.
    - The PivotChart link must be `[Book.xlsx]Sheet!PivotTable1`, or Excel shows an ordinary chart
      (file-format-parity.md, 2026-10-16).
  - Its own gaps.md admits all of this is "unverified in Excel", and the round trips cover only its own
    files.

## 4. Others

- **EPPlus** (github.com/EPPlusSoftware/EPPlus). Commit `0ed726f`.
  - Licence: Polyform Noncommercial 1.0.0, so use it for ideas only. Its test workbooks are under the same
    licence: **do not vendor them**.
  - Since v7.2 it can calculate a pivot: `ExcelPivotTable.Calculate(refresh)`, `CalculatedData`,
    `GetPivotData` and the GETPIVOTDATA worksheet function. It computes values and does not render cells.
  - Code: `src/EPPlus/Table/PivotTable/Calculation/`. `Functions/PivotFunction*.cs` has the 11 aggregates.
    Errors propagate: errors are keyed separately, and `ExcelErrorValue` wins. `ShowDataAs/*.cs` has all 14
    modes (Rank sequential, no ties, `PivotShowAsRankBase.cs`). `Filters/PivotTableFilterMatcher.cs` and
    `PivotCalculationStore.cs` hold the rest.
  - Wiki ("Calculating Pivot tables"): supports page fields, caption/numeric/date/value/top filters,
    slicers and calculated fields. It does not support timelines, OLAP/external sources or the data model,
    and only Default or None subtotals.
  - Test workbooks: `src/EPPlusTest/Workbooks/PivotTableCalculation.xlsx` (Excel-saved; has calcField,
    date and numeric groups), `PivotTableRefreshFields.xlsx` and `PivotTableCF.xlsx`.
- **ClosedXML** (MIT). Commit `057f597`.
  - It reads, edits and writes pivot definitions and caches but does not compute: it writes
    `refreshOnLoad` (`ClosedXML/Excel/PivotTables/*`, 38 files; `IO/PivotTableCacheDefinitionPartWriter.cs`).
  - `ClosedXML/Excel/PivotTables/README.md` lists invariants that make Excel repair or crash, useful for
    our export:
    - `dataFields` ⇔ `pivotField@dataField`, and the axis attribute must match `rowFields`, `colFields` and
      `pageFields`.
    - An axis field with no `<items>` (not even a `default` item) crashes Excel.
    - Σ Values (`-2`) on an axis with one data field crashes Excel; with it, `dataPosition` is required.
    - Page fields need room above `location`.
  - Test resources: 49 workbooks with pivots (118 tables), 38 of them Excel-saved; 20 are copies of
    LibreOffice files.
- **Apache POI** (Apache-2.0). Commit `b8ed3af` (trunk; the suite pins `9958a2a`).
  - `XSSFPivotTable` creates definitions only: row and column labels, data fields, page filters. It never
    computes and never renders the cells.
  - `test-data/spreadsheet` has 7 workbooks with pivots (12 tables). 5 are Excel-saved:
    `ExcelPivotTableSample.xlsx`, `WithChartSheet.xlsx` (5 pivots, tabular, dataOnRows, a PivotChart),
    `54436.xlsx`, `LIBRE_OFFICE-116306-0.xlsx` (14 data fields) and `DataTableCities.xlsx`.
    `LIBRE_OFFICE-128382-0.xlsx` was written by ONLYOFFICE, and `64450.xlsx` has no app.xml.
- **openpyxl** (MIT; hosted on foss.heptapod.net, not cloned). The `openpyxl.pivot` package (`table.py`,
  `cache.py`, `record.py`) round-trips pivot parts as typed objects. It does not compute or render. Useful
  only as a reference for the schema object model.
- **SheetJS.** The Community Edition has no pivot support (0 pivot paths in the `SheetJS/sheetjs` tree,
  rev `3f44ddd`). Pivots are in the commercial Pro edition only.
- **Univer** (Apache-2.0, `dream-num/univer` rev `4348dfa`). The open-source tree has no pivot engine:
  the pivot plugin is the closed Univer Pro. `packages/engine-formula/src/functions/lookup/getpivotdata/index.ts`
  implements GETPIVOTDATA heuristically, by scanning rendered cells (a 200×50 scan limit, `"Grand Total"`,
  a `" TOTAL"` suffix and `"sum of "`… prefixes). That is a label-matching approach SPEC-0015 rightly avoids.
- **IronCalc upstream** (rev `5759341`). No pivot support. GETPIVOTDATA is documented as "not yet
  available" (`docs/.../getpivotdata.md`).
- **HyperFormula** (rev `187dfa1`). No pivots or GETPIVOTDATA.
- **Commercial docs only:**
  - **Aspose.Cells** separates `PivotCache.Refresh` / `RefreshData` (re-read the source; obsolete in newer
    versions) from `PivotTable.CalculateData()` (re-render the table from the cache), with
    `Workbook.RefreshAll()` doing both. Forum threads report renders that differ from Excel's (off-by-one
    rows, column overflow), which shows how hard layout parity is.
    https://docs.aspose.com/cells/net/refresh-pivot-table/
  - **Syncfusion XlsIO**: `PivotTable.Layout()` fills `PivotEngineValues`, a grid of `PivotValueCollections`
    (rows × columns), in `PivotTableLayout`. It has no precise semantics documentation.
  - **SpreadJS (MESCIUS/GrapeCity)** documents compact, outline and tabular, `fillDownLabels` (not in
    compact), `insertBlankLineAfterEachItem`, per-field `subtotalVisible`, and tabular subtotals fixed at
    the bottom. Its claim that compact and outline subtotals default to the bottom is *their* product's
    default; Excel's default is the top. https://developer.mescius.com/spreadjs/docs/features/pivot-table/pivot-table-layout

## 5. Test corpora: Excel-authored xlsx with pivot tables

Counts come from unzipping each file and listing `xl/pivotTables/pivotTableN.xml`. "Excel-saved" means
docProps `Application` is Microsoft Excel or Excel Online. Every such file carries the rendered pivot
cells as cached values, so they are free evidence of Excel's layout and values. Many also carry
`pivotCacheRecords`, so a refresh can be compared with the cells.

| Corpus | Licence | Workbooks with pivots / of which Excel-saved | Pivot tables | Notes |
|---|---|---|---|---|
| Calipers `verification/cases` (rev 9e47ea4) | MIT; pv_goldens from a non-public source, fetched not vendored | 13 goldens / 13 | 19 | Already a RustSheet suite (`eval/suites/calipers`, gate=false). 5 inits also have pivots. |
| Apache POI `test-data/spreadsheet` | Apache-2.0 | 7 / 5 | 12 | Already the `poi` suite (gate=true); 1 PivotChart (`WithChartSheet.xlsx`). |
| SpreadsheetBench 912 (`corpus-bench` suite, 1_*_input) | CC-BY-SA-4.0 | 4 of 912 / 3 | 11 | 41601, 48357, 52379 (1 pivot, 15,858 rows), 59932. Date and field groups. The suite does not trust these caches (`use_cached_values=false`). The full tarball has 24 files with pivots, still the same 4 tasks. |
| IronCalc `calc_tests` (`ironcalc-calc` suite) | MIT/Apache-2.0 | 0 of 26 | 0 | |
| RustSheet's own `crates/rustsheet-xlsx/tests/` | ours | 2 (`pivot_tables.xlsx`, `example.xlsx`) | 3 | |
| LibreOffice `sc/qa/unit/data` (rev b1653ce) | MPL-2.0 (test files) | 70 / 64 (Excel or Excel Online) | 96 | **Best layout corpus**; 1 PivotChart (`forum-mso-de-104083.xlsx`). See the list below. |
| ClosedXML `ClosedXML.Tests/Resource` (rev 057f597) | MIT | 49 / 38 | 118 | 20 are copies of LO files. Includes "input/output" pairs written by ClosedXML: the outputs are not Excel. |
| EPPlus `src/EPPlusTest/Workbooks` | Polyform NC (**do not vendor**) | 3 / 3 | 12 | `PivotTableCalculation.xlsx` covers calcFields and groups. |
| mog test corpus | Apache-2.0 | 1 / 0 (openpyxl and injected XML) | 1 | Not evidence. |
| gridcraft | n/a | 0 | 0 | |

Features covered across the Excel-saved files (from `*.piv`):
- All three forms: compact, outline (`pivottable_outline_mode.xlsx`, `pivottable_duplicated_member_filter.xlsx`)
  and tabular (`pivottable_tabular_mode.xlsx`, `WithChartSheet.xlsx`, `pv23_045`), plus classic drop zones.
- Subtotals at the bottom, no subtotals, custom subtotals, and no grand totals.
- `dataOnRows`, hidden items, label and value `<filters>`, and sort ascending and descending.
- `showDataAs` index and percent.
- Date grouping: years (10 files), quarters (6), months (7), days (1). Numeric `rangePr` and manual
  `Group1` groups.
- Calculated fields: `pivot_calcfield_nameerror.xlsx` (Excel 2010), EPPlus.
- Consolidation and external caches (ClosedXML `PivotTable-AllSources-input.xlsx`).

**Missing** from every corpus: rank show-values-as, difference / % of / running total, `insertBlankRow`,
calculated items, and Excel 365 (lastEdited=7) files with value filters and Top-10 together. These need
oracle probes.

**LibreOffice Excel-saved pivot files** (path under `sc/qa/unit/data/`, pivot count in brackets):
xlsx/pivottable_long_text.xlsx(1) xlsx/tdf122471.xlsx(1) xlsx/pivottable_1s_difference.xlsx(1) xlsx/pivottable_bool_field_filter.xlsx(1) xlsx/pivottable_rowcolpage_field_filter.xlsx(1) xlsx/forum-mso-de-104083.xlsx(1) xlsx/pivottable_double_field_filter.xlsx(3) xlsx/pivot-table/PivotTableCellFormatsTest_8_DataFieldInRow_DataColor.xlsx(1) xlsx/pivot-table/PivotTableWithNoSourceData.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_9_MultipleSelections.xlsx(1) xlsx/pivot-table/Pivot_Table_with_Cell_Protection.xlsx(1) xlsx/pivot-table/PivotTableFormatsGrandTotal.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_13_SelectionInLabelAndData.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_11_WholeDataColumnSelected.xlsx(1) xlsx/pivot-table/many-fields-in-cache.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_10_FormatDefinitionNotMatchingPivotTable.xlsx(1) xlsx/pivot-table/shared-group-field.xlsx(2) xlsx/pivot-table/test_diff_aggregation.xlsx(2) xlsx/pivot-table/PivotTableCellFormatsTest_3_DataFieldInColumn_ColumnLabelColor.xlsx(1) xlsx/pivot-table/first_header_row_zero.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_15_TwoDataFieldColumns_WholeDataColumnSelected.xlsx(1) xlsx/pivot-table/pivot_calcfield_nameerror.xlsx(1) xlsx/pivot-table/shared-dategroup.xlsx(2) xlsx/pivot-table/PivotTableCellFormatsTest_4_DataFieldInColumn_DataColor.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_12_WholeLabelColumnSelected.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_7_TwoRowTwoColumnFields_DataColor.xlsx(1) xlsx/pivot-table/shared-numgroup.xlsx(2) xlsx/pivot-table/PivotTableFormatsSubTotals.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_1_DataFieldInRow_RowLabelColor.xlsx(1) xlsx/pivot-table/two-data-fields.xlsx(1) xlsx/pivot-table/pivotcompact.xlsx(1) xlsx/pivot-table/shared-nested-dategroup.xlsx(2) xlsx/pivot-table/PivotTableCellFormatsTest_14_TwoRowsDataFieldInColumn_LabelColor.xlsx(1) xlsx/pivot-table/with-strings-integers-and-dates.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_2_DataFieldInRow_ColumnLabelColor.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_5_DataFieldInColumnAndTwoRowFields_DataColor.xlsx(1) xlsx/pivot-table/tdf126858-1.xlsx(1) xlsx/pivot-table/PivotTableCellFormatsTest_6_SingleDataFieldInColumn_DataColor.xlsx(1) xlsx/tdf112106.xlsx(1) xlsx/pivottable_tabular_mode.xlsx(1) xlsx/PivotTable_CachedDefinitionAndDataNotInSync_SheetColumnsRemoved_WithCacheData.xlsx(1) xlsx/tdf112936.xlsx(1) xlsx/pivottable_invalid_formats.xlsx(1) xlsx/pivot/Pivot1_Row_Grand_Subtotals.xlsx(1) xlsx/pivot/Pivot1_Row_Grand.xlsx(1) xlsx/pivot/Pivot4_Column_Grand_Subtotals_SortDescending.xlsx(1) xlsx/pivot/Pivot2_Row_Grand_Subtotals.xlsx(1) xlsx/pivot/Pivot3_Column_Grand_Subtotals.xlsx(1) xlsx/pivot/Pivot2_Row_Subtotals_SortDescendingAll.xlsx(1) xlsx/pivot/Pivot1_Row.xlsx(1) xlsx/pivot/Pivot2_Row_Subtotals.xlsx(1) xlsx/pivot/Pivot2_Row.xlsx(1) xlsx/pivot/Pivot2_Row_Grand.xlsx(1) xlsx/pivot/Pivot2_Row_Compact.xlsx(1) xlsx/pivottable_string_field_filter.xlsx(1) xlsx/pivottable_outline_mode.xlsx(1) xlsx/pivottable_error_item_filter.xlsx(1) xlsx/PivotTable_CachedDefinitionAndDataNotInSync_SheetColumnsRemoved_WithoutCacheData.xlsx(1) xlsx/tdf89139_pivot_table.xlsx(1) xlsx/PivotTable_CachedDefinitionAndDataInSync.xlsx(1) xlsx/pivottable_duplicated_member_filter.xlsx(1) xlsx/pivottable_date_field_filter.xlsx(1) xlsx/column-style-autofilter.xlsx(17) xlsx/pivot_dark1.xlsx(1) 

**ClosedXML Excel-saved pivot files** (path under `ClosedXML.Tests/Resource/`, pivot count in brackets):
TryToLoad/SheetsWithoutRelId.xlsx(1) TryToLoad/PivotTableWithTableSource.xlsx(1) TryToLoad/PivotTableWithBorder.xlsx(1) TryToLoad/LoadPivotTables.xlsx(3) TryToLoad/DialogSheet.xlsx(1) TryToLoad/PivotTableWithMissingReference.xlsx(1) TryToLoad/LO/xlsx/pivottable_bool_field_filter.xlsx(1) TryToLoad/LO/xlsx/pivottable_rowcolpage_field_filter.xlsx(1) TryToLoad/LO/xlsx/pivottable_double_field_filter.xlsx(3) TryToLoad/LO/xlsx/pivot-table/many-fields-in-cache.xlsx(1) TryToLoad/LO/xlsx/pivot-table/shared-group-field.xlsx(2) TryToLoad/LO/xlsx/pivot-table/shared-dategroup.xlsx(2) TryToLoad/LO/xlsx/pivot-table/shared-numgroup.xlsx(2) TryToLoad/LO/xlsx/pivot-table/two-data-fields.xlsx(1) TryToLoad/LO/xlsx/pivot-table/shared-nested-dategroup.xlsx(2) TryToLoad/LO/xlsx/pivot-table/with-strings-integers-and-dates.xlsx(1) TryToLoad/LO/xlsx/tdf112106.xlsx(1) TryToLoad/LO/xlsx/pivottable_tabular_mode.xlsx(1) TryToLoad/LO/xlsx/pivottable_string_field_filter.xlsx(1) TryToLoad/LO/xlsx/pivottable_outline_mode.xlsx(1) TryToLoad/LO/xlsx/pivottable_error_item_filter.xlsx(1) TryToLoad/LO/xlsx/tdf89139_pivot_table.xlsx(1) TryToLoad/LO/xlsx/pivottable_duplicated_member_filter.xlsx(1) TryToLoad/LO/xlsx/pivottable_date_field_filter.xlsx(1) TryToLoad/LO/xlsx/column-style-autofilter.xlsx(17) TryToLoad/TemplateWithTableSourcePivotTables.xlsx(14) Other/PivotTable/Sources/PivotTable-AllSources-input.xlsx(7) Other/PivotTable/Save/Pivot_table_conditional_format.xlsx(1) Other/PivotTableReferenceFiles/PivotCacheWithoutSourceData-input.xlsx(1) Other/PivotTableReferenceFiles/VariousDataTypesInTableColumns/input.xlsx(1) Other/PivotTableReferenceFiles/ClearPivotTableRenderedRangeWhenLoading/inputfile.xlsx(1) Other/PivotTableReferenceFiles/PivotMixedDataTypesInTableColumn/input.xlsx(5) Other/PivotTableReferenceFiles/PivotTableWithoutSourceData-input.xlsx(1) Other/PivotTableReferenceFiles/PivotSubtotalsSource/input.xlsx(1) Other/PivotTableReferenceFiles/PivotTableWithNoneTheme/inputfile.xlsx(1) Other/PivotTableReferenceFiles/TwoPivotTablesWithSingleSource/input.xlsx(2) Other/PivotTableReferenceFiles/ChartsheetAndPivotTable.xlsx(1) Other/PivotTableReferenceFiles/VersioningAttributes/inputfile.xlsx(1) 

Fetching: the LibreOffice set needs only a sparse checkout:
`git clone --depth 1 --filter=blob:none --sparse https://github.com/LibreOffice/core` then
`git sparse-checkout set --no-cone 'sc/qa/unit/data/**/*.xlsx'`. That is 365 xlsx files, 8.5 MB. **Do
not** run `git ls-tree -l` on a blob-less clone: it fetches every blob.

## 6. Sources
- https://github.com/fundamental-research-labs/mog, https://github.com/fundamental-research-labs/calipers,
  https://github.com/storytold/gridcraft (commits above)
- https://github.com/EPPlusSoftware/EPPlus/wiki/Calculating-Pivot-tables
- https://github.com/EPPlusSoftware/EPPlus/wiki/Calculating-Pivot-tables-and-the-GetPivotData-Method
- https://docs.aspose.com/cells/net/refresh-pivot-table/
- https://forum.aspose.com/t/issue-on-pivot-table-rendering-after-refresh/115236
- https://help.syncfusion.com/cr/xamarin/Syncfusion.XlsIO.Implementation.PivotTables.PivotTableLayout.html
- https://syncfusion.com/forums/131573/how-to-use-ipivottables-pivotenginevalues-property
- https://developer.mescius.com/spreadjs/docs/features/pivot-table/pivot-table-layout
