#!/usr/bin/env python3
"""Writes the workbook of the probe request (README.md): two sheets that hold the same cells, `L` with
`sheetPr transitionEvaluation="1"` and `E` without, each formula of README.md's table on both, in column Q from
row 2, its id beside it in column P. The formula cells hold no cached value. It also writes the control: the
same package with the attribute on neither sheet.

Standard library only, and the same bytes at every run. Usage: python3 -I generate.py <output directory>
"""
import re
import sys
from pathlib import Path
from xml.sax.saxutils import escape

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, styles, workbook_relationships  # noqa: E402
from probe_package import write_package  # noqa: E402

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
    '<sheets><sheet name="L" sheetId="1" r:id="rId1"/><sheet name="E" sheetId="2" r:id="rId2"/></sheets>'
    '<definedNames><definedName name="OnE">E!$A$1+1</definedName>'
    '<definedName name="OnL">L!$A$1+1</definedName></definedNames>'
    '<calcPr calcId="191029" fullCalcOnLoad="1"/></workbook>'
)

# The cells both sheets hold beside the formulas. A value is text, a number, a Boolean or an error; "" is a
# cell that holds the empty string, and a missing address an empty cell (A6).
INPUTS = {
    "A1": "abc", "A2": "12", "A3": "", "A4": 5, "A5": True, "A7": "ABC", "A8": "#N/A", "A9": "abd",
    "D1": "Name", "E1": "Qty", "D2": "Apple", "E2": 1, "D3": "Apricot", "E3": 2, "D4": "apple", "E4": 4,
    "D5": "Ap", "E5": 8,
    "G1": "Name", "G2": "Ap", "H1": "Name", "H2": "Apple", "I1": "Name", "I2": "apple", "J1": "Qty", "J2": ">1",
    "K1": "Name", "K2": "=Ap", "L1": "Name", "L2": "A*",
    "N1": "12", "O1": 1, "N2": "apple", "O2": 2, "N3": "Banana", "O3": 3,
    "B1": " ", "B2": "3/4", "B3": "1E3", "B4": "TRUE", "B5": "0", "B6": "5",
    "S1": 5, "T1": "five", "S2": 7, "T2": "seven",
    "U1": "Item", "V1": "Val", "U2": "a", "V2": 1, "U3": "ab", "V3": 2, "U4": 5, "V4": 4, "U5": 10, "V5": 8,
    "U6": "", "V6": 16, "V7": 32,
    "W1": "Item", "W2": "a*", "X1": "Item", "X2": "=a", "Y1": "Item", "Y2": "<>", "Z1": "Item", "Z2": ">5",
    "AA1": "Crit",
}
# The two input cells that hold a formula: a cell that calculates the empty string, and a computed criterion.
FORMULA_INPUTS = {"B7": '""', "AA2": 'U2="a"'}


def table_rows() -> list[tuple[str, str]]:
    """(id, formula) for every row of README.md's table, in its order."""
    readme = (Path(__file__).resolve().parent / "README.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| ([A-Z]\d+) \| `=(.*)` \| .* \|$", readme, flags=re.MULTILINE)
    if not rows or len({row[0] for row in rows}) != len(rows):
        sys.exit("README.md does not list the rows of the table, each id once")
    return rows


def cell(address: str, value: object) -> str:
    if value is True or value is False:
        return f'<c r="{address}" t="b"><v>{int(value)}</v></c>'
    if isinstance(value, int | float):
        return f'<c r="{address}"><v>{value!r}</v></c>'
    if value == "#N/A":
        return f'<c r="{address}" t="e"><v>#N/A</v></c>'
    text = escape(str(value))
    space = ' xml:space="preserve"' if text != text.strip() else ""
    return f'<c r="{address}" t="inlineStr"><is><t{space}>{text}</t></is></c>'


def column_number(address: str) -> int:
    letters = re.match(r"[A-Z]+", address).group(0)  # type: ignore[union-attr]
    number = 0
    for ch in letters:
        number = number * 26 + ord(ch) - 64
    return number


def sheet(other: str, transition: bool) -> str:
    """The sheet's cells in row and column order: the inputs, and the table in P and Q from row 2."""
    cells: dict[int, dict[int, str]] = {}
    for address, value in INPUTS.items():
        row = int(re.search(r"\d+", address).group(0))  # type: ignore[union-attr]
        cells.setdefault(row, {})[column_number(address)] = cell(address, value)
    for address, formula in FORMULA_INPUTS.items():
        row = int(re.search(r"\d+", address).group(0))  # type: ignore[union-attr]
        cells.setdefault(row, {})[column_number(address)] = f'<c r="{address}"><f>{escape(formula)}</f></c>'
    for i, (row_id, formula) in enumerate(table_rows()):
        row = i + 2
        cells.setdefault(row, {})[16] = cell(f"P{row}", row_id)
        cells.setdefault(row, {})[17] = f'<c r="Q{row}"><f>{escape(formula.replace("OTHER", other))}</f></c>'
    rows = "".join(
        f'<row r="{row}">' + "".join(cells[row][column] for column in sorted(cells[row])) + "</row>"
        for row in sorted(cells)
    )
    properties = '<sheetPr transitionEvaluation="1"/>' if transition else ""
    return f'{HEAD}<worksheet xmlns="{MAIN}">{properties}<sheetData>{rows}</sheetData></worksheet>'


def write(path: Path, transition: bool) -> None:
    """The workbook, with the attribute on sheet L or, for the control, on neither sheet."""
    write_package(
        path,
        (
            ("[Content_Types].xml", content_types(2, styles=True)),
            ("_rels/.rels", PACKAGE_RELATIONSHIPS),
            ("xl/workbook.xml", WORKBOOK),
            ("xl/_rels/workbook.xml.rels", workbook_relationships(2, styles=True)),
            ("xl/styles.xml", styles()),
            ("xl/worksheets/sheet1.xml", sheet("E", transition=transition)),
            ("xl/worksheets/sheet2.xml", sheet("L", transition=False)),
        ),
    )


def main() -> None:
    out = output_directory()
    # The control first: if Excel refuses it, the package is at fault and the attribute is not yet asked.
    write(out / "transition-evaluation-control.xlsx", transition=False)
    write(out / "transition-evaluation.xlsx", transition=True)


if __name__ == "__main__":
    main()
