"""Build linked-name-legacy.xlsx from eval/suites/fixtures/cases/external-link-cached-ranges.xlsx.

Usage: python3 -I make_probe.py <external-link-cached-ranges.xlsx> <out.xlsx>

Keeps every part of the source but two: the external link part gains three workbook-scoped names
(NR_Row = 'Uses'!$A$1:$C$1, NR_Col = 'Uses'!$A$1:$A$3, NR_One = 'Uses'!$B$1), and sheet Refs is
replaced by the legacy formulas below (plain <f>, no cm attribute, placeholder cached 0).
"""
import sys
import zipfile
from xml.sax.saxutils import escape

SRC, OUT = sys.argv[1], sys.argv[2]
ROW = "[1]!NR_Row"
ROW_W = "[1]Uses!$A$1:$C$1"
COL = "[1]!NR_Col"
COL_W = "[1]Uses!$A$1:$A$3"

# Section 1: the column name, read in rows 2 and 3 (on its rows: A2 5, A3 TRUE) and 9 (off).
# One shape a column, the name and the range written in its place side by side.
COLUMN_SHAPES = [
    ("B", COL), ("C", COL_W),
    ("D", f'{COL}&""'), ("E", f'{COL_W}&""'),
    ("F", f"SUM({COL}+0)"), ("G", f"SUM({COL_W}+0)"),
    ("H", f"IF(TRUE,{COL})"), ("I", f"IF(TRUE,{COL_W})"),
    ("J", f"CHOOSE(1,{COL})"), ("K", f"CHOOSE(1,{COL_W})"),
    ("L", f"INDEX({COL},0)"), ("M", f"INDEX({COL_W},0)"),
]
COLUMN_ROWS = [2, 3, 9]

# Section 2: the row name, read in columns B and C (on its columns: B1 10, C1 #N/A) and E (off).
# One shape a row, label in column A.
ROW_SHAPES = [
    ("name", ROW), ("written", ROW_W),
    ("name&", f'{ROW}&""'), ("written&", f'{ROW_W}&""'),
    ("SUM(name+0)", f"SUM({ROW}+0)"), ("SUM(written+0)", f"SUM({ROW_W}+0)"),
    ("IF arm name", f"IF(TRUE,{ROW})"), ("IF arm written", f"IF(TRUE,{ROW_W})"),
    ("IF else name", f"IF(FALSE,0,{ROW})"), ("IF else written", f"IF(FALSE,0,{ROW_W})"),
    ("IFS name", f"_xlfn.IFS(TRUE,{ROW})"), ("IFS written", f"_xlfn.IFS(TRUE,{ROW_W})"),
    ("CHOOSE name", f"CHOOSE(1,{ROW})"), ("CHOOSE written", f"CHOOSE(1,{ROW_W})"),
    ("INDEX0 name", f"INDEX({ROW},0)"), ("INDEX0 written", f"INDEX({ROW_W},0)"),
    ("SUM(IF) name", f"SUM(IF(TRUE,{ROW}))"), ("SUM(IF) written", f"SUM(IF(TRUE,{ROW_W}))"),
    ("SINGLE name", f"_xlfn.SINGLE({ROW})"), ("SINGLE written", f"_xlfn.SINGLE({ROW_W})"),
    ("one-cell name", "[1]!NR_One"), ("one-cell written", "[1]Uses!$B$1"),
    ("ERROR.TYPE name", f"ERROR.TYPE({ROW})"), ("ERROR.TYPE written", f"ERROR.TYPE({ROW_W})"),
]
ROW_COLUMNS = ["B", "C", "E"]
FIRST_ROW_SHAPE = 12


def formula_cell(ref: str, formula: str) -> str:
    return f'<c r="{ref}"><f>{escape(formula)}</f><v>0</v></c>'


def label_cell(ref: str, text: str) -> str:
    return f'<c r="{ref}" t="inlineStr"><is><t>{escape(text)}</t></is></c>'


rows: dict[int, list[str]] = {}
rows.setdefault(1, []).append(label_cell("A1", "column name NR_Col in rows 2, 3, 9"))
for row in COLUMN_ROWS:
    cells = rows.setdefault(row, [label_cell(f"A{row}", f"row {row}")])
    cells += [formula_cell(f"{column}{row}", shape) for column, shape in COLUMN_SHAPES]
rows.setdefault(11, []).append(label_cell("A11", "row name NR_Row in columns B, C, E"))
for offset, (label, shape) in enumerate(ROW_SHAPES):
    row = FIRST_ROW_SHAPE + offset
    cells = rows.setdefault(row, [label_cell(f"A{row}", label)])
    cells += [formula_cell(f"{column}{row}", shape) for column in ROW_COLUMNS]
sheet = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" '
    'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheetData>'
    + "".join(f'<row r="{row}">{"".join(cells)}</row>' for row, cells in sorted(rows.items()))
    + "</sheetData></worksheet>"
)

NAMES = (
    "<definedNames>"
    "<definedName name=\"NR_Col\" refersTo=\"='Uses'!$A$1:$A$3\"/>"
    "<definedName name=\"NR_Global_B2\" refersTo=\"='Defines'!$B$2\"/>"
    "<definedName name=\"NR_One\" refersTo=\"='Uses'!$B$1\"/>"
    "<definedName name=\"NR_Range\" refersTo=\"='Uses'!$A$1:$B$3\"/>"
    "<definedName name=\"NR_Row\" refersTo=\"='Uses'!$A$1:$C$1\"/>"
    "<definedName name=\"NR_Local_Range\" refersTo=\"='Defines'!$A$1:$B$2\" sheetId=\"1\"/>"
    "<definedName name=\"NR_To_A1\" refersTo=\"='Defines'!$A$1\" sheetId=\"1\"/>"
    "</definedNames>"
)
with zipfile.ZipFile(SRC) as source, zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as out:
    for item in source.infolist():
        data = source.read(item.filename)
        if item.filename == "xl/worksheets/sheet1.xml":
            data = sheet.encode()
        elif item.filename == "xl/externalLinks/externalLink1.xml":
            # The source's four names, kept, and the three new ones, workbook names sorted as
            # Excel writes them and the sheet's own after them, as in the source.
            text = data.decode()
            head, rest = text.split("<definedNames>", 1)
            _, tail = rest.split("</definedNames>", 1)
            data = (head + NAMES + tail).encode()
        out.writestr(item, data)
