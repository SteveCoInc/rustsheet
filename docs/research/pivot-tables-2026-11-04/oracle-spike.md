# Oracle spike: can excel-graph render pivots we author? (2026-11-04)

Live excel-graph runs, made with a throwaway script and not committed. The script does the following:
1. Builds a probe workbook with spec2xlsx from `probes/pivot-rendered-cells`.
2. Mutates its package.
3. Uploads it and opens a session.
4. Reads ranges through Graph on open, after `calculate(FullRebuild)` and after
   `worksheets(...)/pivotTables/refreshAll`.
5. Saves through the usual scratch-sheet edit and downloads Excel's file.

These are **observations, not committed evidence.** Plan task 2 turns each one into a probe case with an
oracle sidecar before anything relies on it (CLAUDE.md, directive 1).

Source data in every run: Data!A1:C7 holds Region/Product/Sales, with rows East A 10, West A 20, East B 5,
West B 7, East C 3 and East A 4.

| Run | Package change | What Excel for the web did |
|---|---|---|
| 1 | Data!C2 changed from 10 to 1000; cache records and rendered cells left stale (10) | **Re-rendered on open:** East/A 1004, East total 1012, grand total 1039, before any calculate or refresh call. It rewrote the cache (`refreshedBy="Excel Services"`, new `sharedItems`) and the definition (`updatedVersion="8"`, x16 `pivotTableDefinition16` ext). Answers T-2327: `refreshOnLoad` pivots are re-rendered. |
| 2 | Cache emptied (`<sharedItems/>`, no records) and a pivot definition with no `<items>` | **Refused to open:** createSession gave 501 `OpenWorkbookBlockedWorkbook` ("not compatible with Excel for the web"). The builder must write a consistent cache, records and axis items. |
| 3 | Hand-written tabular definition (two row fields, Values on columns, three data fields, `showDataAs="percentOfTotal"`), no `rowItems`/`colItems`, builder's old rendered cells left in place | The anchor A3 showed **`#SPILL!`** and the pivot did not render, because the new layout would have overwritten cells the pivot did not own. This matches the documented 2025 "PivotTable #SPILL!" behaviour (excel-docs.md, PSPILL). |
| 4 | Run 3 with the target rows emptied | **Rendered in full:** `Values` / `Region Product Sum of Sales Pct Count of Sales`, then East A/B/C, `East Total` 22 0.4489… 4, West A/B, `West Total` 27, `Grand Total` 49 1 6. Excel wrote `location ref="A3:E12"`, eight `rowItems` (`<i r="1">`, `t="default"`, `t="grand"`) and three `colItems` with `i=` indices. |
| 5 | Run 4 plus a calculated cache field `Double = Sales*2+0.1` (`databaseField="0"`), `sortType="descending"` on Product, and a `top10 val="2"` count filter on Product by Sum of Sales | **Rendered.** East: B 5 / 10.1, then A 14 / 28.1, total 19 / 38.1. West: B 7 / 14.1, then A 20 / 40.1, total 27 / 54.1. Grand 46 / 92.1. The **calculated field applies to the sums** (East/A 14·2+0.1 = 28.1, against 28.2 over records), the **Top-N filter changes the subtotals** (East 19, not 22; C dropped), and the label sort is descending. |
| 6 | A Date-like number column grouped numerically: `fieldGroup` with `rangePr startNum=45000 endNum=45400 groupInterval=100` and our own group labels `45000-45099`… | **Rendered with Excel's own labels:** `45000-45100` 39, `45100-45200` 7, `45300-45400` 3, Grand Total 49. Empty groups and `<`/`>` bins were not shown. Excel replaced the labels we supplied, so the oracle teaches group naming. (The data here are fractional, 45031.5; integer data may label bins `a-(a+interval-1)`, as LibreOffice's notes say. Probe both.) |

Conclusions for the plan:
- **Authoring works.** A spec case can give the source data and any pivot definition. Excel computes the
  layout, the values, the group items, `rowItems`/`colItems` and the location. Neither Graph (list, get,
  refresh and refreshAll only) nor Office.js is needed.
- **The minimum Excel needs:**
  - a cache consistent with the source (shared items, records, flags);
  - `<items>` for every axis field;
  - a target area holding no foreign cells.
- **`refreshAll` adds nothing after open.** Every value was already final on open.
- **Graph's `values` read is display-rounded** (0.285714285714286). The downloaded file holds full precision
  and is what sidecars use, through the snapshot reader.
- The pivot area's cells are **constants** in the saved file. Today's sidecars record formula cells only,
  hence plan task 5.
