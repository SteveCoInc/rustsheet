#!/usr/bin/env python3
"""Writes the workbook of the probe request (README.md): `review-moves.xlsx`.

Sheet `Moves` holds A1:D10, so that a cut, a copy, a sort and a row delete each leave a cell that tells where
its row went: column A counts down from 10 to 1 (a sort by it, smallest first, turns the rows over), and B, C
and D name their own first place (`B2`, `C3`). Sheet `Long` holds one text of 32,767 characters, the longest a
cell holds, to paste into a note. No comment and no note is in the file: the request's steps add them in Excel.

Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, styles, workbook_relationships  # noqa: E402
from probe_package import write_package  # noqa: E402

ROWS = 10
# The longest text a cell holds: 32,766 times `a`, then `z`, so that a text cut short has no `z`.
LONG = "a" * 32766 + "z"

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}"><sheets>'
    '<sheet name="Moves" sheetId="1" r:id="rId1"/><sheet name="Long" sheetId="2" r:id="rId2"/>'
    "</sheets></workbook>"
)


def text(address: str, value: str) -> str:
    return f'<c r="{address}" t="inlineStr"><is><t>{value}</t></is></c>'


def moves() -> str:
    """A1:D10: a number that counts down in A, and each other cell's own address."""
    rows = "".join(
        f'<row r="{row}"><c r="A{row}"><v>{ROWS + 1 - row}</v></c>'
        + "".join(text(f"{column}{row}", f"{column}{row}") for column in "BCD")
        + "</row>"
        for row in range(1, ROWS + 1)
    )
    return f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData>{rows}</sheetData></worksheet>'


def long() -> str:
    return f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData><row r="1">{text("A1", LONG)}</row></sheetData></worksheet>'


def main() -> None:
    out = output_directory()
    write_package(
        out / "review-moves.xlsx",
        (
            ("[Content_Types].xml", content_types(2, styles=True)),
            ("_rels/.rels", PACKAGE_RELATIONSHIPS),
            ("xl/workbook.xml", WORKBOOK),
            ("xl/_rels/workbook.xml.rels", workbook_relationships(2, styles=True)),
            ("xl/styles.xml", styles()),
            ("xl/worksheets/sheet1.xml", moves()),
            ("xl/worksheets/sheet2.xml", long()),
        ),
    )


if __name__ == "__main__":
    main()
