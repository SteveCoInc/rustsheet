# Probe request `circular-references`: which cells Excel names for a circular reference

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For Taskei P559284750
and SPEC-0002 Design 14, which gives the engine two lists (the cells on a loop, and the cells that only read
one) for the Circular References warning, the status bar and the menu. Microsoft documents that the three exist
and not which cells they name (`docs/excel-semantics/recalc.md`, "Which cells Excel names for a circular
reference is documented in part, and not probed"). Design 14 marks each rule that waits for a case with it.*

## Why by hand

No API reads the list in Excel for the web: Office.js has none, and `Worksheet.CircularReference` is VBA. So
each case is a person's reading of three things in Excel for the web, the set of record (decisions 0266 and
0039):

- **the warning**: the dialog Excel shows when a circular reference is created, or its absence;
- **the status bar**: its text, with the cell address if it shows one;
- **the menu**: Formulas > Error Checking > Circular References, every address it lists, in its order.

Not all three may exist there. Microsoft's page ("Remove or allow a circular reference in Excel", read
2026-11-12) gives the menu for the Windows and Mac desktops and says: "Excel for the web and mobile apps:
Formula auditing commands can be limited. Open the file in a desktop app to locate and fix circular
references." No desktop Excel fills probes (the project lead's ruling of 2026-11-12), so a reading Excel for the
web does not have is an answer: please write "no status bar text" or "no such menu", and use the search box
for "Circular References" once before writing the second. A case whose readings do not exist has no oracle,
and its rule is then entered as `NO-ORACLE` on Microsoft's page and on Design 14 (decision 0464).

## How

Each case starts in a **new workbook**, in Automatic mode with iterative calculation off unless it says
otherwise. Manual mode is Formulas > Calculation Options > Manual, and "F9" below is Calculate Workbook on the
same tab. Iterative calculation has no documented switch in Excel for the web's ribbon; if there is none, turn
it on for the workbook with Script Lab (Office.js `Application.iterativeCalculation`, ExcelApi 1.9):
`await Excel.run(async (c) => { c.workbook.application.iterativeCalculation.enabled = true; await c.sync(); });`
Type the formulas in the order given: the order is part of the question. After each step marked
*read*, write down the three readings. Please return them as `RESULTS.md` beside this file, one heading a case,
with the build of Excel; a reading that is not there ("no address", "the menu is greyed out") is an answer.

| Case | Steps | Questions |
|---|---|---|
| C1 | A1 `=B1`, then B1 `=A1`; *read*. In a second new workbook: B1 `=A1`, then A1 `=B1`; *read* | Which cell does the status bar name? Does the typing order change it? Which cells does the menu list, in which order? |
| C2 | as C1's first workbook, then D1 `=A1+1`; *read* | Is D1, which only reads the loop, in the menu? Does the status bar change? |
| C3 | A1 `=B1`, B1 `=A1`, then A5 `=B5`, B5 `=A5`; *read* | Does the menu list both loops, and every cell of each or one a loop? Which cell does the status bar name? |
| C4 | on Sheet2: A1 `=B1`, B1 `=A1`; go to Sheet1; *read*; go back to Sheet2; *read* | The status bar's text on each sheet; the menu on each sheet |
| C5 | turn iterative calculation on; A1 `=B1`, B1 `=A1`; *read* | Is there a warning? What do the status bar and the menu show? |
| C6 | A1 `=B1`, B1 `=A1` (close the warning); then A5 `=B5`, B5 `=A5`; *read*. Undo twice, redo twice; *read*. Close and reopen the workbook; *read* | Is the warning shown for the second loop? When an undo and redo bring a loop back? When the workbook is opened again? |
| C7 | Manual mode; A1 `=B1+1`; F9; then B1 `=A1`; *read*; F9; *read* | Is the warning shown when B1 is entered, or at F9? What do A1 and B1 show in between? |
| C8 | A1 `=SEQUENCE(A2+2)` (A2 is empty, and A1's spill covers it), then C1 `=A1+1`; *read*. In a second new workbook, the same with iterative calculation on; *read* | What do A1 and C1 show? Is A1 in the menu, and C1? The status bar's text |
| C9 | Manual mode; A1 `=A1+1`; *read*; F9; *read* | Is the warning shown at the entry? Is A1 in the menu before F9? |
| C10 | B1 `=C1+D1`, then C1 `=B1`, then D1 `=C1`; *read* | Does the menu list D1, which is on the loop through B1's second operand? In which order are the three listed? |
| C11 | A1 0, then B1 `=IF(A1=0,7,B1)`; *read*; A1 = 1; *read*; A1 = 0; *read* | Is there a warning when B1 is entered, while its formula does not take the branch that reads itself? When A1 becomes 1? Is B1 still named after A1 is 0 again, and what does B1 show at each step? |

## What the answers decide

Each answer becomes a rule of `docs/excel-semantics/recalc.md` under the entry named above, with its case as
evidence, and the mark in Design 14 comes off. The provisional rules, each stated there: the menu lists a loop's
members and not its readers (C2, C10); members are in sheet, row and column order and the status bar names the
first on the active sheet, with no address when all are elsewhere (C1, C3, C4); with iterative calculation on
there is no warning (C5); ordinary iteration-cycle cells are in neither list, but driver-marked circular
anchors and their readers remain listed (C8); a loop is the reads a calculation makes, so a formula that names
itself in a branch it does not take is on no loop until it takes it (C11); a new loop warns once per session
(C6); a formula typed in Manual mode that refers to itself is reported when it is entered, and a loop closed
through a cell the entry did not write at the next calculation (C7, C9). An answer that contradicts a
provisional rule changes the rule, not the answer.
