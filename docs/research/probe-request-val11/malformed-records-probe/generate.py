#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md): three packages that each hold one record form the
importer bounds (a row's record repeated, a `<col>` over columns an earlier one covers, a hyperlink whose `ref`
is written backwards), and for each a control that differs only in being well formed.

Standard library only, and the same bytes at every run. Usage: python3 -I generate.py <output directory>
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, styles, workbook_relationships  # noqa: E402
from probe_package import write_package  # noqa: E402

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
    '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>'
    '<calcPr calcId="191029" fullCalcOnLoad="1"/></workbook>'
)
# Cell format 1 is the built-in number format 2 ("0.00") and cell format 2 the built-in 10 ("0.00%"): a row
# record's `s` is seen in what an empty cell of the row shows once a number is typed into it.
STYLES = styles(cell_formats=(2, 10))

# Row 1 and row 3 are ordinary. D1:D4 are the formulas whose values say what Excel made of row 2:
# its two cells, whether SUBTOTAL and AGGREGATE see it as hidden, and the sheet's sum.
FORMULAS = (
    '<c r="D1"><f>A2+B2</f></c><c r="E1"><f>SUBTOTAL(109,A1:A3)</f></c>'
    '<c r="F1"><f>_xlfn.AGGREGATE(9,5,A1:A3)</f></c><c r="G1"><f>SUM(A1:B3)</f></c>'
)
ROW_1 = f'<row r="1"><c r="A1"><v>1</v></c>{FORMULAS}</row>'
ROW_3 = '<row r="3"><c r="A3"><v>100</v></c></row>'
FIRST_RECORD = '<row r="2" ht="30" customHeight="1" s="1" customFormat="1"><c r="A2"><v>10</v></c></row>'
SECOND_RECORD = (
    '<row r="2" ht="60" customHeight="1" hidden="1" s="2" customFormat="1">'
    '<c r="A2"><v>20</v></c><c r="B2"><v>5</v></c></row>'
)
# The control holds what the importer makes of the two records: the first record's height, format and
# visibility, and the cells as the second record leaves them (measured on main 621a8367b: A2 = 20, B2 = 5, and
# the row not hidden).
AS_IMPORTED = (
    '<row r="2" ht="30" customHeight="1" s="1" customFormat="1">'
    '<c r="A2"><v>20</v></c><c r="B2"><v>5</v></c></row>'
)
ROWS = {
    "row-repeated.xlsx": ROW_1 + FIRST_RECORD + SECOND_RECORD + ROW_3,
    "row-repeated-control.xlsx": ROW_1 + AS_IMPORTED + ROW_3,
}

# Columns A to E at width 20, then B and C again at width 40 and hidden. The control gives B and C to the
# first record alone, as the importer does.
COLS = {
    "col-overlap.xlsx": (
        '<cols><col min="1" max="5" width="20" customWidth="1"/>'
        '<col min="2" max="3" width="40" customWidth="1" hidden="1"/></cols>'
    ),
    "col-overlap-control.xlsx": '<cols><col min="1" max="5" width="20" customWidth="1"/></cols>',
}
# No formula can say a column's width or whether it is hidden on Excel for the web (`CELL("width")` is `#VALUE!`
# there, docs/excel-semantics/information.md): the answer is in the `<cols>` Excel writes back.
COL_CELLS = (
    '<row r="1"><c r="A1"><v>1</v></c><c r="B1"><v>2</v></c><c r="C1"><v>3</v></c>'
    '<c r="G1"><f>SUM(A1:C1)</f></c></row>'
)

# A link to a place in the workbook, over the range written from its bottom-right corner to its top-left one.
# The control writes the same range the usual way.
LINKS = {
    "hyperlink-reversed.xlsx": '<hyperlinks><hyperlink ref="B2:A1" location="S!D5" display="x"/></hyperlinks>',
    "hyperlink-reversed-control.xlsx": (
        '<hyperlinks><hyperlink ref="A1:B2" location="S!D5" display="x"/></hyperlinks>'
    ),
}
LINK_CELLS = (
    '<row r="1"><c r="A1"><v>1</v></c><c r="B1"><v>2</v></c><c r="G1"><f>SUM(A1:B2)</f></c></row>'
    '<row r="2"><c r="A2"><v>3</v></c><c r="B2"><v>4</v></c></row>'
)


def sheet(cols: str, rows: str, links: str) -> str:
    return f'{HEAD}<worksheet xmlns="{MAIN}">{cols}<sheetData>{rows}</sheetData>{links}</worksheet>'


def main() -> None:
    out = output_directory()
    sheets = {name: sheet("", rows, "") for name, rows in ROWS.items()}
    sheets.update({name: sheet(cols, COL_CELLS, "") for name, cols in COLS.items()})
    sheets.update({name: sheet("", LINK_CELLS, links) for name, links in LINKS.items()})
    for name, xml in sheets.items():
        write_package(
            out / name,
            (
                ("[Content_Types].xml", content_types(1, styles=True)),
                ("_rels/.rels", PACKAGE_RELATIONSHIPS),
                ("xl/workbook.xml", WORKBOOK),
                ("xl/_rels/workbook.xml.rels", workbook_relationships(1, styles=True)),
                ("xl/styles.xml", STYLES),
                ("xl/worksheets/sheet1.xml", xml),
            ),
        )


if __name__ == "__main__":
    main()
