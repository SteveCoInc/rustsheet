"""Build cse-row-column-area.xlsx from eval/suites/fixtures/cases/valueless-cell-forms.xlsx.

Usage: python3 -I make_probe.py <valueless-cell-forms.xlsx> <out.xlsx>

Keeps the package's parts and replaces the shared strings and sheet Sheet1 with legacy (CSE) array
formulas over areas (<f t="array" ref=..>, no cm attribute), placeholder cached 0, and their inputs.
"""
import sys
import zipfile
from xml.sax.saxutils import escape

SRC, OUT = sys.argv[1], sys.argv[2]
STRINGS = ["k", "m", "a", "b", "c", "d"]
# A1:B4 keys and items, D1 the key.
VALUES = {"A1": "k", "A2": "m", "A3": "k", "A4": "k", "B1": "a", "B2": "b", "B3": "c", "B4": "d", "D1": "k"}
# (anchor, area, formula): each a CSE array formula over its area.
ARRAYS = [
    ("F1", "F1:F4", 'IFERROR(INDEX($B$1:$B$4,SMALL(IF($A$1:$A$4=$D$1,ROW($A$1:$A$4)),ROW())),"")'),
    ("G1", "G1:G3", "ROW()"),
    ("H5", "H5:J5", "COLUMN()"),
    ("K2", "K2:L3", "ROW()*10+COLUMN()"),
    ("A7", "A7", "ROW()"),
    ("C7", "C7:C9", "ROWS(ROW())"),
    ("D7", "D7:F7", "COLUMNS(COLUMN())"),
    ("G7", "G7:G9", "SUM(ROW())"),
    ("H7", "H7:H9", "ROW()-6"),
]
# Plain formulas reading the areas back, one a row, in column N.
READERS = [f"_xlfn.FORMULATEXT({anchor})" for anchor, _, _ in ARRAYS]


def column_letters(ref: str) -> str:
    return "".join(ch for ch in ref if ch.isalpha())


def row_number(ref: str) -> int:
    return int("".join(ch for ch in ref if ch.isdigit()))


def cells_of(area: str) -> list[str]:
    first, _, last = area.partition(":")
    last = last or first
    def col(letters: str) -> int:
        n = 0
        for ch in letters:
            n = n * 26 + ord(ch) - 64
        return n
    def letters(n: int) -> str:
        s = ""
        while n:
            n, r = divmod(n - 1, 26)
            s = chr(65 + r) + s
        return s
    c1, c2 = col(column_letters(first)), col(column_letters(last))
    r1, r2 = row_number(first), row_number(last)
    return [f"{letters(c)}{r}" for r in range(r1, r2 + 1) for c in range(c1, c2 + 1)]


cells: dict[str, str] = {}
for ref, text in VALUES.items():
    cells[ref] = f'<c r="{ref}" t="s"><v>{STRINGS.index(text)}</v></c>'
for anchor, area, formula in ARRAYS:
    for ref in cells_of(area):
        if ref == anchor:
            cells[ref] = f'<c r="{ref}"><f t="array" ref="{area}">{escape(formula)}</f><v>0</v></c>'
        else:
            cells[ref] = f'<c r="{ref}"><v>0</v></c>'
for index, formula in enumerate(READERS):
    ref = f"N{index + 1}"
    cells[ref] = f'<c r="{ref}" t="str"><f>{escape(formula)}</f><v></v></c>'

def sort_key(ref: str):
    letters = column_letters(ref)
    return (row_number(ref), len(letters), letters)

rows: dict[int, list[str]] = {}
for ref in sorted(cells, key=sort_key):
    rows.setdefault(row_number(ref), []).append(cells[ref])
sheet = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>'
    + "".join(f'<row r="{r}">{"".join(c)}</row>' for r, c in sorted(rows.items()))
    + "</sheetData></worksheet>"
)
shared = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    f'<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="{len(STRINGS)}" '
    f'uniqueCount="{len(STRINGS)}">' + "".join(f"<si><t>{s}</t></si>" for s in STRINGS) + "</sst>"
)
with zipfile.ZipFile(SRC) as source, zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as out:
    for item in source.infolist():
        data = source.read(item.filename)
        if item.filename == "xl/worksheets/sheet1.xml":
            data = sheet.encode()
        elif item.filename == "xl/sharedStrings.xml":
            data = shared.encode()
        out.writestr(item, data)
