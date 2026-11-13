# D-MODEL-91: a table's own AutoFilter (Excel for the web, en-US)

From poc-model-cc, 2026-11-12; rewritten the same day for Excel for the web (Steve's ruling, 07:20Z: no desktop
Excel). Every step uses commands Excel for the web has: Insert > Table, Data > Filter, the Table Design tab's Header
Row and Filter Button, Insert > Slicer, Sort, Clear and Reapply. Where a step saves a file, use File > Save As >
Download a Copy and read its parts. Step 3 uploads a hand-edited file and opens it. No formula cells: what Excel does
with a table's AutoFilter beside the sheet's. The model ships interim rules until these are observed (in brackets). Each answer is a sentence and, where a
file is saved, the `xl/tables/table1.xml` and `xl/worksheets/sheet1.xml` `<autoFilter>`/`<sortState>` elements.

Setup for every step: a new workbook; A1:D11 holds a header row `Cat, Val, X, Y` and ten data rows (`Cat` a,b,a,c,a,b,a,c,a,b; `Val` 1-10; X and Y any text).

1. **Insert > Table over a filtered range.** Data > Filter on A1:D11, keep `Cat` = a (rows 3, 5, 7, 9, 11 hidden).
   Then select A1 and Insert > Table (My table has headers). Does Excel allow it? Is the sheet AutoFilter gone, and does
   the table's own filter keep `Cat` = a, or are all rows shown? Save; record both parts' `<autoFilter>`.
   [interim: the model refuses to apply a sheet AutoFilter whose range meets a table; creating a table is unchanged]
2. **Data > Filter next to a table.** Undo to the plain range. Make F1:G6 a table (header `K, L`, five rows). Select A1
   (A1:D11's current region does not reach F) and Data > Filter: which range does the sheet AutoFilter take? Then
   remove it, put values in E1:E11 so A1's current region reaches the table, select A1 and Data > Filter again: the
   range Excel takes, or the message it shows (its exact text).
3. **A file with both.** Save a workbook whose sheet `<autoFilter ref="A1:G11">` overlaps a table at F1:G6 (edit the
   sheet XML by hand). Does Excel open it, repair it (the repair log's text), and what is left?
4. **Header Row off.** A table A1:D11 with `Cat` = a applied (five rows hidden). Table Design > Header Row off: do the
   rows stay hidden? Header Row on again: are the buttons back with `Cat` = a still applied?
   [interim: with no header row shown the table's filter hides nothing and its criteria are kept]
5. **Filter Button off.** The same filtered table. Table Design > Filter Button off: do the hidden rows show? Filter
   Button on again: is `Cat` = a back, or no criterion? Data > Filter (active cell in the table) does the same?
   [interim: Filter Button off removes the filter and shows its rows]
6. **Sort state.** The table unfiltered: `Val` dropdown > Sort Largest to Smallest. Save; record the table part's
   `<sortState>` (inside or beside its `<autoFilter>`) and whether the sheet part has one. Then Data > Clear: does the
   sort state go?
7. **A slicer on a table whose Filter Button goes off** (from poc-pivot-ui-cc). A table A1:D11, Insert > Slicer on
   `Cat`, select `a` in the slicer (rows hidden). Table Design > Filter Button off: do the rows stay hidden, and does
   the slicer still filter when another item is clicked? Then Filter Button on again: does the `Cat` button show
   the slicer's criterion? (Tells whether the buttons are a view of the table's filter or the filter itself.)
   [interim: Filter Button off removes the filter; the slicer of such a table is off until the button is on]
8. **Header Row off, then Data > Filter** (from poc-ui-c-cc). A table A1:D11, Table Design > Header Row off, the
   active cell in it, Data > Filter (and Ctrl+Shift+L, where the browser passes it): does Excel turn the header row back on, refuse with a message
   (its exact text), or do nothing? [UI interim, D-UI-194: refused, "Show the table's header row to filter it"]
9. **Banding in a filtered table.** The default style (Banded Rows on), `Cat` = a applied: do the visible rows
   alternate bands, show no band, or do two banded rows sit together where a hidden row was? [UI today: banding counts
   hidden rows]
10. **Clear and Reapply beside a table.** A sheet AutoFilter on F1:G6 with a criterion, beside the filtered table.
    Active cell in the table, Data > Clear: which filter clears, the table's or both? Then the active cell in F,
    Data > Reapply: does only the sheet's apply? [UI interim: each acts on the table holding the active cell, else
    on the sheet's]
