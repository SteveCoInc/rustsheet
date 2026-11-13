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
| package-name-scoped-to-no-sheet.xlsx | defined name `Lost` has localSheetId 5 in a workbook of 2 sheets (Data, Calc); `Kept` is Data-scoped, `Global` workbook-scoped | Calc!A1 `=Lost`, A2 `=ISREF(Lost)`, A3 `=Global` (9), A4 `=Data!Kept` (8), A5 `=Data!A1*2` (14); also: does Name Manager list `Lost`, and with what scope? |
| package-table-part-missing.xlsx | the sheet's table relationship targets ../tables/table1.xml, which the zip lacks | Data!B1 `=A1*2` (A1 7) |

Calc's questions (Data!A1 = 7, Data!B1 = 1; Gone is very hidden):
A1 `=SHEETS()` (3 if Gone is kept, 2 if dropped), A2 `=SHEET("Calc")` (3 or 2), A3 `=ISREF(Gone!A1)` (TRUE if
kept), A4 `=Gone!A1+1` (1 if kept empty), A5 `=Data!A1*2` (14), A6 `=SHEETS(Data:Calc)` (3 or 2), A7
`=SUM(Data:Calc!B1)` (1). Also wanted: the sheet list Excel shows (Workbook.Worksheets, hidden included).

What the engine gives today (branches poc/cfix-loadref-sheets and -styles): shapes 1 to 3 load, Gone kept as an
empty very hidden sheet. Checked in the engine's tests: A1 `=SHEETS()` 3, A4 1, A5 14, A7 1. A6 is `#N/IMPL!`
(SHEETS of a 3-D span is not implemented); A2 and A3 not checked. Shapes 4 and 5 are still refused ("Zip Error:
specified file not found in archive").

Added 2026-11-12 12h00Z (coordinator ruling on CR-322034591): package-name-scoped-to-no-sheet.xlsx
(`python3 -I make_probes.py <dir> names`). The engine (poc/cfix-loadref-localname) skips `Lost` and notes it, so
Calc!A1 and A2 give #NAME? and FALSE. If Excel keeps the name, for example as workbook-scoped, that becomes a
follow-up fix.

Added 2026-11-12 13h15Z, after review of the CRs (`python3 -I make_probes.py <dir> round2`). **The importer's rules
changed in review; this supersedes the "What the engine gives today" paragraph above:** shapes 2 and 3 (an `r:id`
that names no relationship, a relationship to a missing worksheet part) are refused again; only a sheet listed with
an EMPTY `r:id` loads empty (the shape of poi/60825.xlsx, a legacy macro sheet). Four more packages:

| File | Shape | Cells to read |
|---|---|---|
| package-sheet-empty-rid.xlsx | sheet Gone (2nd of Data, Gone, Calc; veryHidden) has `r:id=""` | Calc!A1:A7 as above; and the sheet list Excel shows |
| package-styles-relocated.xlsx | the styles part is xl/formatting/custom.xml, named by the workbook's styles relationship; no xl/styles.xml | Data!A1 (0.5, style 1 = percent in that part): shown as 50%?; B1 `=CELL("format",A1)` (P0 if the part is read, G if not) |
| package-styles-dangling-relationship.xlsx | the styles relationship names xl/formatting/custom.xml, which is missing; a valid xl/styles.xml (style 1 = percent) is there, named by nothing | does it open? if so A1 and B1 as above (P0 means Excel read xl/styles.xml by its name) |
| package-row-style-past-cellxfs.xlsx | row 1 has s="5" customFormat="1" and column C style="7"; cellXfs has 1 entry | does it open? Data!B1 `=CELL("format",A1)`, D2 `=CELL("format",C2)` (G expected) |

What the engine gives (the lane's integration tree, `rustsheet-corpus load-file` on each package): empty-rid,
relocated and row-style load; dangling-relationship is refused ("Zip Error: specified file not found in archive").
By the branches' tests of the same shapes: Gone loads empty and very hidden, `SHEETS()` 3; the relocated part's
styles are the ones read (a percent style 0 shows 0.5 as 50%); the row's and the column's style read as no style,
noted in the import report.

Added 2026-11-12 20h40Z (`python3 -I make_probes.py <dir> round3`): package-row-style-negative.xlsx, the row 9 of the
10 corpus files hold, `<row r="1" ht="30" customHeight="1" s="-2147483648" customFormat="1">`, in a workbook of its
own (a reader that takes `s` as unsigned may refuse it before it compares it with cellXfs). Does Excel open it, and
what are row 1's height and Data!B1 `=CELL("format",A1)` (A1 0.5)? The engine loads it, the row with no style and
its height 30, noted (unknown-record-style).
