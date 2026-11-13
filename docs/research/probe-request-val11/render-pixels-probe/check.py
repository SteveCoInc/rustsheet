#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the workbook to a scratch directory and reads it back. It says nothing about how Excel draws it:
that is what the request asks. Checked here:
- the workbook's sha256 is the one README.md names;
- it holds the sheets `Borders` and `Text`, in that order;
- on `Borders`, B2, D2, F2, H2 and J2 are boxed in thin, medium, thick, double and dashed, in the order
  README.md's table gives, each named in the cell above it, B4:J4 have a thin line under them, and no other
  cell has a format;
- on `Text`, A1:A5 hold the five lines README.md lists, in one black Calibri 11, and only A2 has a fill.
"""
import sys
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import MAIN, fail, generated  # noqa: E402

NS = {"m": MAIN}
NAME = "render-pixels.xlsx"
BOXES = {"B2": "thin", "D2": "medium", "F2": "thick", "H2": "double", "J2": "dashed"}
LINES = {
    "A1": "Hamburgefonstiv 0123456789",
    "A2": "Hamburgefonstiv 0123456789",
    "A3": "illiill WAVE To AV fi fl",
    "A4": "abcdefghijklmnopqrstuvwxyz",
    "A5": "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
}


def formats(styles: bytes) -> list[tuple[dict[str, str | None], str | None]]:
    """For each cell format: its border's four side styles, and its fill's colour if it is solid."""
    root = ElementTree.fromstring(styles)
    borders = [
        {side: b.find(f"m:{side}", NS).get("style") for side in ("left", "right", "top", "bottom")}
        for b in root.findall("m:borders/m:border", NS)
    ]
    fills = []
    for fill in root.findall("m:fills/m:fill", NS):
        pattern = fill.find("m:patternFill", NS)
        colour = pattern.find("m:fgColor", NS)
        fills.append(colour.get("rgb") if pattern.get("patternType") == "solid" and colour is not None else None)
    return [(borders[int(xf.get("borderId"))], fills[int(xf.get("fillId"))]) for xf in root.findall("m:cellXfs/m:xf", NS)]


def cells(xml: bytes) -> dict[str, tuple[int, str]]:
    """address -> (cell format, inline text) for every cell of a sheet part."""
    out = {}
    for c in ElementTree.fromstring(xml).iterfind(".//m:c", NS):
        inline = c.find("m:is/m:t", NS)
        out[c.get("r")] = (int(c.get("s", "0")), inline.text if inline is not None else "")
    return out


def main() -> None:
    readme = " ".join((HERE / "README.md").read_text(encoding="utf-8").split())
    packages = generated(HERE)
    if list(packages) != [NAME]:
        fail(f"generate.py wrote {list(packages)}, not {NAME} alone")
    sha, parts = packages[NAME]
    book = ElementTree.fromstring(parts["xl/workbook.xml"])
    if [sheet.get("name") for sheet in book.iterfind(".//m:sheet", NS)] != ["Borders", "Text"]:
        fail("the sheets are not Borders and Text")
    table = formats(parts["xl/styles.xml"])
    fonts = ElementTree.fromstring(parts["xl/styles.xml"]).findall("m:fonts/m:font", NS)
    if len(fonts) != 1 or fonts[0].find("m:name", NS).get("val") != "Calibri" or fonts[0].find("m:sz", NS).get("val") != "11":
        fail("the one font is not Calibri 11")
    borders = cells(parts["xl/worksheets/sheet1.xml"])
    for address, style in BOXES.items():
        sides, fill = table[borders[address][0]]
        if set(sides.values()) != {style} or fill is not None:
            fail(f"Borders!{address} is not boxed in {style}: {sides}")
        if borders.get(f"{address[0]}1") != (0, style):
            fail(f"Borders!{address[0]}1 does not name {style}")
        if f"| `{address}` | {style} |" not in readme:
            fail(f"README.md's table does not give {address} as {style}")
    for column in "BCDEFGHIJ":
        if f"{column}4" not in borders:
            fail(f"Borders!{column}4 is not a cell: the thin line runs under B4:J4")
        sides, fill = table[borders[f"{column}4"][0]]
        if sides != {"left": None, "right": None, "top": None, "bottom": "thin"} or fill is not None:
            fail(f"Borders!{column}4 does not have a thin line under it alone: {sides}")
    if len(borders) != 5 + 5 + 9:
        fail(f"Borders holds {len(borders)} cells, not the 19 README.md describes")
    lines = cells(parts["xl/worksheets/sheet2.xml"])
    if {a: t for a, (_, t) in lines.items()} != LINES:
        fail("Text!A1:A5 do not hold the five lines")
    for address, (fmt, line) in lines.items():
        sides, fill = table[fmt]
        if any(sides.values()) or (fill is not None) != (address == "A2"):
            fail(f"Text!{address}: border {sides}, fill {fill}")
        if f"`{line}`" not in readme:
            fail(f"README.md does not list the line {line}")
    # Last, so that a change of the workbook is first told by what changed in it.
    if f"`{NAME}` (sha256 `{sha}`)" not in readme:
        fail(f"README.md does not name {NAME} with its sha256 {sha}")
    print(f"ok: {NAME}, five boxed cells, a thin line under nine cells, five lines of Calibri 11")


if __name__ == "__main__":
    main()
