# cfix-missing-parts-probe (poc-cfix-loadref-cc, 2026-11-12)

Five hand-built packages (make_probes.py writes them; Application "RustSheet fixture", every cached value a
placeholder 0). Each lacks one part a well-formed workbook has. **The answer wanted first is whether Excel opens
each one** (excel-graph: createSession succeeds, or 501 `FileCorruptTryRepair` / `unsupportedWorkbook`, which is
itself the answer). Where it opens, the values. Upload each file as it is; the canary embedder may not read
these packages (T-0112 did the same for poi/49609.xlsx). Desktop Excel, if anyone has it to hand: does it open
silently or offer a repair, and what does the repair log say? (M14 question, decision 0039.)

Why: the full corpus run refuses 400 + 5 files of shape 2 (Taskei P560810770, P560917917) and 151 of shape 1
(P558955276). The importer now loads them, by the rules in docs/excel-semantics/xlsx-storage.md "A sheet with no
part loads empty" and "A package may have no styles part", both UNVERIFIED until this probe answers.

| File | What is missing | Cells to read |
|---|---|---|
| package-no-styles-part.xlsx | no xl/styles.xml, no styles relationship | Data!B1 `=LEN(A1)` (A1 inline "abc"), B2 `=A2*2` (A2 5), B3 `=ISTEXT(A1)` |
| package-sheet-without-relationship.xlsx | sheet Gone (2nd of Data, Gone, Calc) has r:id="rId9", which the workbook rels lack | Calc!A1:A7 below |
| package-sheet-part-missing.xlsx | sheet Gone's relationship rId9 targets worksheets/sheet9.xml, which the zip lacks | Calc!A1:A7 below |
| package-comments-part-missing.xlsx | the sheet's comments relationship targets ../comments1.xml, which the zip lacks | Data!B1 `=A1*2` (A1 7) |
| package-table-part-missing.xlsx | the sheet's table relationship targets ../tables/table1.xml, which the zip lacks | Data!B1 `=A1*2` (A1 7) |

Calc's questions (Data!A1 = 7, Data!B1 = 1; Gone is very hidden):
A1 `=SHEETS()` (3 if Gone is kept, 2 if dropped), A2 `=SHEET("Calc")` (3 or 2), A3 `=ISREF(Gone!A1)` (TRUE if
kept), A4 `=Gone!A1+1` (1 if kept empty), A5 `=Data!A1*2` (14), A6 `=SHEETS(Data:Calc)` (3 or 2), A7
`=SUM(Data:Calc!B1)` (1). Also wanted: the sheet list Excel shows (Workbook.Worksheets, hidden included).

What the engine gives today (branches poc/cfix-loadref-sheets and -styles): shapes 1 to 3 load, Gone kept as an
empty very hidden sheet. Checked in the engine's tests: A1 `=SHEETS()` 3, A4 1, A5 14, A7 1. A6 is `#N/IMPL!`
(SHEETS of a 3-D span is not implemented); A2 and A3 not checked. Shapes 4 and 5 are still refused ("Zip Error:
specified file not found in archive").
