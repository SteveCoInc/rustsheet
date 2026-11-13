# cf-relative-wrap-probe (poc-engine-c-cc, 2026-11-12, Taskei P559198894; review of CR-322219666, B3)

One workbook. Office.js (a rendered fill), or by eye in Excel for the web or desktop.

**Question.** A conditional format's formula is written for the top-left cell of its range and read in every cell of
it. A rule "the same as the cell above" applied from row 1 is stored `A1048576=A1`. Does Excel wrap the relative row
round the grid in rows 2 to 6 (so row 2 compares A1 with A2), as desktop Excel 16.x does for a relative defined name
(references.md "Relative references in defined names")?

**Input.** `cf-relative-wrap.xlsx`, sha256 `b67dacedc0a63a4b349b996ee25365725690f608f24243f0e3b229ae06cf5128`, built
by `make_probe.py` from the repository's `eval/suites/fixtures/cases/valueless-cell-forms.xlsx`. Sheet1 A1:A6 = 1, 1,
2, 2, 3, 3; one rule on A1:A6, expression `A1048576=A1`, a red fill (dxf 0). B1:B6 hold the same comparison as a
plain formula per row (`=A1048576=A1` in B1, `=A1=A2` in B2, ...) as a value control: B1 FALSE, B2 TRUE, B3 FALSE,
B4 TRUE, B5 FALSE, B6 TRUE.

**Wanted.** For A1:A6, whether each cell shows the red fill (`Range.getDisplayedCellProperties` format.fill.color,
ExcelApi 1.19, or a picture).

**Engine's answers.** Main before CR-322219666: no cell red (a relative row past the grid read as empty). That CR's
branch: A2, A4 and A6 red, as the B column's values.
