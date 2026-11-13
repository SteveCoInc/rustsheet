#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the two workbooks to a scratch directory and reads them back. It says nothing about what Excel does
with them: that is what the request asks. Checked here, against the two tables of README.md:
- the files are the two the README lists, each with the sha256 it gives, every part well-formed XML and every
  relationship's target a part of the package (`listed`, in probe_package.py);
- the shared strings are `zero`, `one`, `two`, `three`;
- the workbook is in Manual mode and does not ask for a calculation on load;
- each cell of column A is stored as the README gives it: its type, its formula and its value.
"""

import re
import sys
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import fail, listed, readme_table  # noqa: E402

MAIN = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"


def stored(cell: ElementTree.Element, namespace: str) -> tuple[str | None, str | None, str | None]:
    """A cell's type, formula and value."""
    return cell.get("t"), cell.findtext(f"{namespace}f"), cell.findtext(f"{namespace}v")


def main() -> None:
    first = [(row[0], row[1].strip("`")) for row in readme_table(HERE, "| Cell | Stored as |")]
    count = 0
    for (name, described, _), trees in listed(HERE, "| File | Column A |"):
        count += 1
        wanted = first if name == "formula-string-value.xlsx" else re.findall(r"(A\d+) `([^`]+)`", described)
        strings = ["".join(item.itertext()) for item in trees["xl/sharedStrings.xml"]]
        if strings != ["zero", "one", "two", "three"]:
            fail(f"{name}: the shared strings are {strings}")
        calculation = trees["xl/workbook.xml"].find(f"{MAIN}calcPr").attrib
        if calculation.get("calcMode") != "manual" or "fullCalcOnLoad" in calculation:
            fail(f"{name}: calcPr is {calculation}")
        cells = list(trees["xl/worksheets/sheet1.xml"].iter(f"{MAIN}c"))
        if [cell.get("r") for cell in cells] != [cell for cell, _ in wanted]:
            fail(f"{name}: the cells are {[cell.get('r') for cell in cells]}")
        for cell, (address, text) in zip(cells, wanted):
            # The README shows the cell as the file has it, without its address.
            if stored(cell, MAIN) != stored(ElementTree.fromstring(text), ""):
                fail(f"{name}: {address} is stored as {stored(cell, MAIN)}, and README.md gives {text}")
    print(f"ok: {count} workbooks as README.md lists them")


if __name__ == "__main__":
    main()
