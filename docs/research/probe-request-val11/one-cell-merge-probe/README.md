# Merging one cell (Excel for the web, en-US)

From poc-model-cc, 2026-11-12. Excel for the web only. The question: does a merge command on a single cell make a
merge? The proof of concept writes no merge record for one cell (D-MODEL-62 (d)), and I-03's reference planner writes
one. Each answer is a sentence, with what the downloaded .xlsx holds (`<mergeCells>` in the sheet part, and A1's
alignment in styles).

1. Type `x` in B2 and select B2 alone. Home > Merge & Center. Is anything merged (does Unmerge Cells become
   available)? Is B2 centred? Download the file: is there a `<mergeCell ref="B2:B2"/>`, or no `<mergeCells>` at all?
2. The same with Merge Across, and with Merge Cells, each on a fresh single cell.
3. Office.js (Automate or Script Lab): `sheet.getRange("C3").merge(false)`, then `sheet.getRange("C3").getMergedAreasOrNullObject()`
   loaded: null, or the area C3? Then `merge(true)` (across) on D4. Any error code and message.
