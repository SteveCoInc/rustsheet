# Pivot feature catalog v1: how it is built and what the sources disagree on (T-4125)

The catalog is `eval/catalog/pivot-features.tsv`. Per decision 0274 it is M16's scope list: a row is
in scope unless a decision rules it out. `tools/src/rustsheet_tools/pivot_catalog.py`
(`rustsheet-pivot-catalog --xsd DIR`) generates it.

## How it is built

- **Schemas.** The generator walks three schemas. They are not vendored: the generator reads them
  from a docx4j checkout pinned at f960df59ef49f0d0a81f2ad7383b7dfaad14c46b (Apache-2.0; the schemas
  themselves are ECMA-376's and Microsoft's [MS-XLSX]):
  - `xsd/sml/sml_ECMA376_4ed_transitional.xsd` (ECMA-376 4th edition, transitional SpreadsheetML);
  - `xsd/xlsx/office_spreadsheetml_2009_9_main.xsd` (x14);
  - `xsd/xlsx/office_spreadsheetml_2010_11_main.xsd` (x15).

  To regenerate:
  ```
  git clone --filter=blob:none --sparse https://github.com/plutext/docx4j
  cd docx4j && git sparse-checkout set xsd/sml xsd/xlsx && git checkout f960df59ef49f0d0a81f2ad7383b7dfaad14c46b
  cd tools && uv run rustsheet-pivot-catalog --xsd ../docx4j/xsd
  ```
- **Roots.**
  - Main schema: `pivotTableDefinition`, `pivotCacheDefinition` and `pivotCacheRecords`.
  - x14: the pivot table, cache, field, data field and hierarchy extensions, plus slicers, slicer
    caches and slicer styles.
  - x15: the pivot extensions, timelines, table slicer caches and timeline styles.

  The walk lists every element and attribute reachable from these roots as a path, with the
  schema's default. It also emits one row per value of the enumerations a pivot computes with:
  ST_DataConsolidateFunction, ST_ShowDataAs, x14 ST_PivotShowAs, ST_PivotFilterType, ST_GroupBy,
  ST_ItemType, ST_FieldSortType and ST_Axis.
- **Size.** The catalog has 1,634 rows:
  - 879 are in scope;
  - 420 are out of scope (OLAP, the Data Model, what-if edits, consolidation sources);
  - 335 are round-trip only (formatting and styles, UI-only flags, slicer and timeline chrome, and
    file metadata), which decision 0274 rules out beyond preservation.
- **Committed copies.** The generator also writes the node list to
  `eval/catalog/pivot-schema-nodes.txt`. `tools/tests/test_pivot_catalog.py` fails when:
  - a node has no row;
  - a row has no node, unless it is `retired`;
  - a row's default disagrees with the schema;
  - a row's family, scope or stage disagrees with the rules.

  It runs without the schemas.
- **Hand-filled columns.** A row's `web`, `status` and `evidence` are filled by probes and
  implementations, and its `owner` (T-4952) names the open M16 task that owns it for the M16 exit row.
  Regeneration keeps them, together with the row's id, so ids are stable. A row
  whose node leaves the walk stays, with status `retired`, so its id is never reused.
- **Classification.** Path-keyword rules in `RULES` set the family, `affects` and stage. The first
  match wins, and no node is left uncategorized. The rules are a heuristic: a row whose probe shows
  otherwise gets its rule corrected, not a hand edit to the TSV, because the test would reject one.

## Reconciliation: schema vs Office.js vs the docs

The Office.js enums come from the ExcelApi reference pages (excel-docs.md §4). The docs are the
feature list in excel-docs.md §1.

1. **Aggregation.**
   - ECMA's ST_DataConsolidateFunction has 11 values.
   - Office.js AggregationFunction has the same 11 under other names (`countNums` = CountNumbers,
     `stdDevp` = StandardDeviationP), plus `Automatic` and `Unknown`. Neither extra has an OOXML
     value. `Automatic` is the field default (sum for numbers, count otherwise), which the file
     spells out as `subtotal`.
   - Distinct Count is in neither. Only the x15 schema has it, as `x15:dataField@isCountDistinct`.
     It is Data Model only, so it is out of scope by decision 0274.
   - Finding: probe what `Automatic` writes for a mixed field (T-4142's aggregate probes).
2. **Show Values As.**
   - ECMA has 9 modes and x14 adds 6. That is 15 modes counting `normal`, and Office.js has the same
     15 plus `unknown`.
   - The Office.js enum spells `rankDecending` (sic).
   - No source documents the default `baseItem` for x14 modes. MS-OI29500 18.10.1.22 gives only the
     magic previous and next values (1048828 and 1048829).
3. **Filters.**
   - ST_PivotFilterType has 66 values.
   - Office.js's label conditions have 9 values. Its `exclusive` flag stands for the 5 `captionNot*`
     values (`captionNotEqual`, `captionNotBetween`, `captionNotBeginsWith`, `captionNotEndsWith` and
     `captionNotContains`).
   - Office.js's value conditions map to the `value*` values and to `count`, `percent` and `sum`
     (TopN and BottomN with `selectionType`).
   - Office.js's date conditions map to the `date*` and relative values and to `M1`–`M12` and
     `Q1`–`Q4`.
   - Findings:
     - **(a)** The schema's `dateNotBetween`, `dateNotEqual`, `valueNotBetween` and
       `valueNotEqual` are Office.js's value and date filters with `exclusive` set (excel-docs.md
       §1.7), as for captions. What Excel for the web renders for them is still unprobed (T-4164).
     - **(b)** The `unknown` value is listed in the schema but has no meaning in any source. The
       catalog keeps it as a row.
4. **Layout.**
   - Office.js's PivotLayoutType (Compact, Outline, Tabular) has no single attribute. It maps to the
     table's `compact`/`outline` with `compactData`/`outlineData` and the per-field
     `compact`/`outline`.
   - SubtotalLocationType (AtTop, AtBottom, Off) maps to `subtotalTop` and `defaultSubtotal`.
   - The catalog lists the attributes. The T-4137/T-4138 probes record the combinations Excel
     renders.
5. **Grouping and calculated items.** These are present in the schema (125 grouping rows, 42
   calculated rows) and absent from Office.js entirely. Only authored files exercise them, so the
   case builder writes them (pivot_definition.py), as for the M6 grouping work.
6. **Things the docs have that the schema does not.**
   - PivotTable Auto Refresh and PivotTable `#SPILL!` (excel-docs.md §0) have no attribute.
   - The `#SPILL!` collapse is VERIFIED in pivots.md and has no catalog row.
   - Finding: the report table (task `report-table`) needs a way to count behaviours that are not
     schema nodes. Proposed: a second, hand-written section of the TSV with ids `PVB-nnnn`. It is
     filed as part of that task, not here.
7. **Defaults the standard states differently from Excel.** The `default` column is the schema's.
   MS-OI29500 inverts or changes these, so the row's probe must assume Excel's meaning (excel-docs.md
   §7):
   - `asteriskTotals` (OLAP, so out of scope; listed for the round trip);
   - `immersive`;
   - `preserveFormatting`;
   - `item@s` (manual filter);
   - `item@sd` (expanded);
   - `indent` (one less, 127 = none);
   - `missingItemsLimit` (absent = automatic);
   - `pivotField@numFmtId` (0);
   - `pivotTableStyleInfo` show* (0).
8. **A schema not covered.**
   - The task names the [MS-XLSX] `xpdl` schema (pivot data model extensions). docx4j has no copy of
     it.
   - Its content is Data Model only (decision 0274: out of scope), so v1 omits it. If a Data Model
     row is ever ruled in, it needs its own fetch.
9. **Slicers and timelines.**
   - They are 114 in-scope rows at stage PV5. Their OLAP slicer cache parts are out of scope. Their
     styles, geometry and chrome (`rowHeight`, `columnCount`, `scrollPosition`, `showHeader` and the
     like) are round-trip only.
   - Office.js's `Slicer.sortBy` maps to the slicer cache's `sortOrder`. `SlicerItem.hasData` is the
     cross-filter state: the tabular cache's `crossFilter` and the item's `i@nd` (excel-docs.md
     §1.14).
