# A rule's formula anchor when its areas are listed out of order (Excel for the web, en-US)

From poc-model-cc and poc-engine-b-cc, 2026-11-12. Excel for the web only. The question: a conditional format's or a
data validation's formula is relative to one cell of the rule's areas. Is that cell the top-left of all its areas
(the engine's rule_anchor, V1, UNVERIFIED), or the first cell of the first area as listed (I-02's conditional-format
record says so)? They differ when the first area listed is not the top-left one. Each answer is the cells that pass,
and what the downloaded .xlsx holds in `sqref` and the formula.

Data: A1:A4 = 1, 2, 3, 4; B1:B4 = 10, 0, 30, 0. Office.js (Automate or Script Lab) is needed to list the areas in an
order the UI would not choose.

1. **Conditional format.** `sheet.getRanges("C3:C4, C1:C2").conditionalFormats.add(Excel.ConditionalFormatType.custom)`,
   with the rule formula `=B1>0` and a fill. Put any value in C1:C4. Which of C1:C4 are filled? (Top-left anchor: C1
   compares B1, so C1 and C3 are filled. First-area anchor: C3 compares B1, so the formula shifts by two rows.) Then
   download the file: the `sqref` order and the stored formula.
2. **Data validation.** `sheet.getRanges("D3:D4, D1:D2").dataValidation.rule = {custom: {formula: "=B1>0"}}`. Type
   5 into each of D1:D4: which are refused? Then download: the `sqref` order and `formula1`.
3. If the web reorders the areas when it saves, say so: that alone answers what a file round trip keeps.
