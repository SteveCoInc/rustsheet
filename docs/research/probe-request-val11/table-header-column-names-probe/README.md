# A table's column names from its header cells (Excel for the web, en-US)

From poc-model-cc, 2026-11-12, asked by poc-pivot-ui-cc. Excel for the web only (Steve's ruling, 2026-11-12). The
question: which text a table column takes from its header cell when the header is not plain text. The model names a
column by the header cell's input, so a header typed `'2023` names its column `'2023`, apostrophe included. Each
answer is a sentence, with the column's name as `=Table1[[#Headers],[...]]` and the Table Design > Table Name and the
structured references show it.

Do each step on a fresh sheet. Type the header row A1:E1 as given, B2:E2 any data, select A1:E2, Insert > Table with
"My table has headers".

1. **Text with the apostrophe prefix.** A1 `'2023`, B1 `'007`, C1 `'=A2`, D1 `'TRUE`, E1 `Plain`. The five column
   names (for example, read `=A1` in G1, and type `=Table1[` in H1 to see the offered names). [model: `'2023`, `'007`,
   `'=A2`, `'TRUE`; expected from Excel: no apostrophe]
2. **A number, a date and a boolean.** A1 `2023`, B1 `1/2/2023` (a date), C1 `TRUE`, D1 `12.50` (formatted 0.00), E1
   `50%`. The five names, and whether the header cells still hold numbers or now hold text (`=ISTEXT(A1)` and so on in
   row 4).
3. **A formula.** A1 `=1+1`, B1 `="Q"&1`, C1 `=NA()`, D1 empty, E1 `=E2`. The five names, and what A1:E1 hold after
   the table is made (a formula, or its value as text).
4. **Typing over a header later.** In step 1's table, type `'2024` into A1, then `2025` (a number) into B1. The names
   after each.
