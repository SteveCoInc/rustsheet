# D-MODEL-93: a sheet named History (Excel for the web, en-US)

From poc-model-cc, 2026-11-12. Excel for the web only (Steve's ruling, 2026-11-12 07:20Z: no desktop Excel). What
Excel does with the sheet name History, typed and in a file. The model ships the rule in brackets until these are
observed. Each answer is a sentence, with the exact text of any message.

1. **Rename to History.** A new workbook. Double-click the Sheet1 tab and type `History`, then Enter. Is it refused,
   and with which message? Try `HISTORY` and `history` too. [model: refused in any letter case]
2. **A file that holds one.** Upload `history-sheet.xlsx` (one sheet named `History`, A1 = 1; any writer that keeps
   the name, e.g. openpyxl or LibreOffice, can make it) and open it. Does it open with the tab named History, or is it
   renamed or repaired (the message's text)? Then rename the tab to `Data` and back to `History`: refused?
   [model: a file's History sheet keeps its name; typing it back is refused]
3. **Office.js.** In the same workbook (Automate > a new script, or the Script Lab add-in), run
   `context.workbook.worksheets.add("History")` and `sheet.name = "History"` on another sheet. Each one's error code
   and message, or none. [model: both refused, as typed names]
4. **Copy of a History sheet.** In the file of step 2, right-click the History tab > Duplicate (or Move or Copy >
   Create a copy). The copy's name. [model: `History (2)`]
