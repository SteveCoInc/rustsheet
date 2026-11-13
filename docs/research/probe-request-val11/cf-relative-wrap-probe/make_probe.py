"""Build cf-relative-wrap.xlsx from eval/suites/fixtures/cases/valueless-cell-forms.xlsx.

Usage: python3 -I make_probe.py <valueless-cell-forms.xlsx> <out.xlsx>

Sheet1 A1:A6 = 1, 1, 2, 2, 3, 3, with one conditional format on A1:A6: the expression
`A1048576=A1` (as a file stores "the same as the cell above" applied from row 1), a red fill.
B1:B6 hold the same test as a plain formula per row (`=A1048576=A1` in B1, relative), as a value
control.
"""
import sys
import zipfile

SRC, OUT = sys.argv[1], sys.argv[2]
rows = []
for row, value in enumerate([1, 1, 2, 2, 3, 3], start=1):
    above = 1048576 if row == 1 else row - 1
    rows.append(
        f'<row r="{row}"><c r="A{row}"><v>{value}</v></c>'
        f'<c r="B{row}" t="b"><f>A{above}=A{row}</f><v>0</v></c></row>'
    )
sheet = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>'
    + "".join(rows)
    + '</sheetData><conditionalFormatting sqref="A1:A6"><cfRule type="expression" dxfId="0" priority="1">'
    "<formula>A1048576=A1</formula></cfRule></conditionalFormatting></worksheet>"
)
DXFS = (
    '<dxfs count="1"><dxf><fill><patternFill patternType="solid"><bgColor rgb="FFFF0000"/>'
    "</patternFill></fill></dxf></dxfs>"
)
with zipfile.ZipFile(SRC) as source, zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as out:
    for item in source.infolist():
        data = source.read(item.filename)
        if item.filename == "xl/worksheets/sheet1.xml":
            data = sheet.encode()
        elif item.filename == "xl/styles.xml":
            text = data.decode()
            assert "</cellStyles>" in text
            data = text.replace("</cellStyles>", "</cellStyles>" + DXFS, 1).encode()
        out.writestr(item, data)
