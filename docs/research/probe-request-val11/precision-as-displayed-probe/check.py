#!/usr/bin/env python3
"""Checks the table files generate.py writes, without Excel: python3 -I check.py

`node check.js` holds each file's sha256 to README.md. This reads what the two table files hold, against the
rows README.md lists under "The rows of the table":
- every row's number is the constant in column A and its formula is in column B, both in the row's format;
- C and D read A and B in General, and no formula cell holds a cached value;
- one Ctrl+Shift+Enter array, `=A2:A3/7`, covers F2:F3, with F2 in the format of row 2 and F3 in General;
- `pad-table.xlsx` says `fullPrecision="0"` and its control does not.
It says nothing about Excel's values: those are what the request asks for.
"""
import re
import sys
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import MAIN, fail, generated  # noqa: E402

NS = {"m": MAIN}


def main() -> None:
    readme = (HERE / "README.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| (\d+) \| `(.*)` \| (\S+) \| `=(.*)` \|$", readme, flags=re.MULTILINE)
    if len(rows) != 99:
        fail(f"README.md lists {len(rows)} rows of the table, not 99")
    if "(`=A2:A3/7`)" not in readme:
        fail("README.md does not give the array's formula as the files hold it, =A2:A3/7")
    packages = generated(HERE)
    for name, precision in (("pad-table.xlsx", "0"), ("pad-table-control.xlsx", None)):
        if name not in packages:
            fail(f"generate.py did not write {name}")
        parts = packages[name][1]
        sheet = ElementTree.fromstring(parts["xl/worksheets/sheet1.xml"])
        styles = ElementTree.fromstring(parts["xl/styles.xml"])
        book = ElementTree.fromstring(parts["xl/workbook.xml"])
        codes = {f.get("numFmtId"): f.get("formatCode") for f in styles.iterfind("m:numFmts/m:numFmt", NS)}
        formats = [codes.get(x.get("numFmtId"), "General") for x in styles.iterfind("m:cellXfs/m:xf", NS)]
        cells = {c.get("r"): c for c in sheet.iterfind(".//m:c", NS)}

        def held(ref: str) -> tuple[str | None, str | None, str, bool]:
            """A cell's formula, its attribute `t`, its format and whether it holds a value."""
            cell = cells.get(ref)
            if cell is None:
                fail(f"{name}: no cell {ref}")
            formula = cell.find("m:f", NS)
            text = None if formula is None else formula.text
            kind = None if formula is None else formula.get("t")
            return text, kind, formats[int(cell.get("s", "0"))], cell.find("m:v", NS) is not None

        for row, code, number, text in rows:
            a, b, c, d = (f"{column}{row}" for column in "ABCD")
            value = cells[a].find("m:v", NS) if a in cells else None
            if value is None or float(value.text) != float(number) or held(a)[2] != code:
                fail(f"{name}: {a} is not the constant {number} in {code}")
            if held(b) != (text, None, code, False):
                fail(f"{name}: {b} is not ={text} in {code} with no cached value")
            for ref, source in ((c, a), (d, b)):
                if held(ref) != (source, None, "General", False):
                    fail(f"{name}: {ref} does not read {source} in General with no cached value")
        array = held("F2") == ("A2:A3/7", "array", rows[0][1], False)
        if not array or cells["F2"].find("m:f", NS).get("ref") != "F2:F3":
            fail(f"{name}: F2 is not the array =A2:A3/7 over F2:F3 in {rows[0][1]}")
        if held("F3") != (None, None, "General", False):
            fail(f"{name}: F3 is not an empty cell in General with no cached value")
        calc = book.find("m:calcPr", NS)
        if calc is None or calc.get("fullPrecision") != precision:
            fail(f"{name}: calcPr fullPrecision is not {precision}")
    print(f"ok: 2 table files, {len(rows)} rows each")


if __name__ == "__main__":
    main()
