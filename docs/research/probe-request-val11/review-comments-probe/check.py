#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the workbook to a scratch directory and reads it back. It says nothing about what Excel does with
it: that is what the request asks. Checked here:
- the workbook's sha256 is the one README.md names;
- it holds the sheets `Moves` and `Long`, in that order, and no comment, note or drawing part;
- on `Moves`, A1:A10 count down from 10 to 1, so that a sort by column A, smallest first, puts row 2 at row 9
  and row 3 at row 8, as README.md says, and every cell of B1:D10 holds its own address;
- on `Long`, A1 is a text of 32,767 characters that ends in `z`.
"""
import sys
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import MAIN, fail, generated  # noqa: E402

NS = {"m": MAIN}
NAME = "review-moves.xlsx"


def cells(xml: bytes) -> dict[str, str]:
    """address -> content, for the numbers and inline texts of a sheet part."""
    out = {}
    for c in ElementTree.fromstring(xml).iterfind(".//m:c", NS):
        value, inline = c.find("m:v", NS), c.find("m:is/m:t", NS)
        out[c.get("r")] = (inline.text or "") if inline is not None else (value.text or "")
    return out


def main() -> None:
    readme = " ".join((HERE / "README.md").read_text(encoding="utf-8").split())
    packages = generated(HERE)
    if list(packages) != [NAME]:
        fail(f"generate.py wrote {list(packages)}, not {NAME} alone")
    sha, parts = packages[NAME]
    book = ElementTree.fromstring(parts["xl/workbook.xml"])
    names = [sheet.get("name") for sheet in book.iterfind(".//m:sheet", NS)]
    if names != ["Moves", "Long"]:
        fail(f"the sheets are {names}")
    extra = [part for part in parts if "comment" in part.lower() or "drawing" in part.lower()]
    if extra:
        fail(f"the workbook holds {extra}: the steps add the comments and notes in Excel")
    moves = cells(parts["xl/worksheets/sheet1.xml"])
    if [moves.get(f"A{row}") for row in range(1, 11)] != [str(n) for n in range(10, 0, -1)]:
        fail("Moves!A1:A10 do not count down from 10 to 1")
    # Sorted by column A, smallest first, row r goes to row 11 - r.
    if "row 2 goes to row 9 and row 3 to row 8" not in readme:
        fail("README.md does not say where a sort puts rows 2 and 3")
    wrong = [a for a in (f"{c}{r}" for c in "BCD" for r in range(1, 11)) if moves.get(a) != a]
    if wrong or len(moves) != 40:
        fail(f"Moves!B1:D10 do not each hold their own address: {wrong[:3]}, {len(moves)} cells")
    long = cells(parts["xl/worksheets/sheet2.xml"])
    text = long.get("A1", "")
    if list(long) != ["A1"] or len(text) != 32767 or not text.endswith("az") or set(text[:-1]) != {"a"}:
        fail(f"Long!A1 is not 32,766 times a and a z: {len(text)} characters")
    # Last, so that a change of the workbook is first told by what changed in it.
    if f"`{NAME}` (sha256 `{sha}`)" not in readme:
        fail(f"README.md does not name {NAME} with its sha256 {sha}")
    print(f"ok: {NAME}, two sheets, 40 cells and one text of {len(text):,} characters")


if __name__ == "__main__":
    main()
