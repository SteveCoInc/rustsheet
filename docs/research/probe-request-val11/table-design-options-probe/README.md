# Table Design: Header Row, Total Row and Resize Table on an existing table (Excel for the web, en-US)

From poc-model-cc, 2026-11-12. Excel for the web only (Steve's ruling, 2026-11-12). The collaborative sheet plans
Table Design > Table Style Options (Header Row, Total Row) and Resize Table (rustsheet-sheet-ops `table_design`,
landing on main as the model lane's Table Design CR). Which totals formula Excel writes for `sum` is on record
(docs/excel-semantics/xlsx-storage.md, "Table parts in the export": `SUBTOTAL(109,Table1[Feb])`). Everything below
is not. Each answer is a sentence or the cells' contents; `[model: ...]` says what the planner does today.

Setup for every step, on a fresh sheet: A1 `Item`, B1 `Qty`, C1 `Price`; A2:A4 `a`, `b`, `c`; B2:B4 `1`, `2`, `3`;
C2:C4 `10`, `20`, `30`. Select A1:C4, Insert > Table, "My table has headers". The table is Table1.

1. **Header Row off.** Table Design > uncheck Header Row. What do A1:C1 hold now? The table's range (Table Design >
   Resize Table shows it). [model: A1:C1 are cleared; the table is A2:C4; the names are kept]
2. **Header Row on again.** After step 1, check Header Row. Which row holds the names, and are they `Item`, `Qty`,
   `Price`? [model: row 1 again, the same names]
3. **Header Row on over input.** After step 1, type `x` into B1, then check Header Row. Does Excel refuse, ask,
   shift cells, or overwrite B1? [model: refused]
4. **Total Row on, numbers last.** Check Total Row. The formulas or values in A5:C5, exactly as the formula bar shows
   them. [model: A5 `Total`, B5 empty, C5 `=SUBTOTAL(109,Table1[Price])`]
5. **Total Row on, text last.** A fresh setup with C2:C4 `p`, `q`, `r` (text). Check Total Row. A5:C5 as in step 4.
   [model: the same as step 4, `sum`]
6. **Total Row on over input.** A fresh setup, type `y` into B5, then check Total Row. Refuse, ask, shift, or
   overwrite? [model: refused]
7. **Total Row off and on.** After step 4, set B5's totals function to Average (its drop-down). Uncheck Total Row: what
   do A5:C5 hold? Check it again: A5:C5 as in step 4. [model: off clears A5:C5; on writes back `Total`, the
   Average formula and the Sum formula]
8. **The other functions.** After step 4, set C5 in turn to each item of its drop-down (Average, Count, Count
   Numbers, Max, Min, Sum, StdDev, Var) and note the formula each writes. [model: SUBTOTAL 101, 103, 102, 104, 105,
   109, 107, 110 with `Table1[Price]`]
9. **Resize with the totals row on.** After step 4, Table Design > Resize Table to A1:D5, then to A1:C6. What does
   Excel do with the totals row each time (allowed, moved, refused)? [model: refused while the totals row is on]
10. **Resize adding columns.** A fresh setup, D1 `Qty` (a name the table has), E1 empty. Resize Table to A1:E4. The
    names of the new columns D and E, and what D1:E1 hold. [model: D shows as `Qty2` while D1 keeps `Qty`, as Insert Table names a repeated header; E is `Column1`,
    written into E1]
