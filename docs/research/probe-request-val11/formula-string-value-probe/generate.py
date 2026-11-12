#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md): formula cells whose cached value is typed as a shared
string (`t="s"`), a form Excel does not write, beside the two forms it does write.

The workbooks are in Manual mode and do not ask for a calculation on load, so that Excel may show the cached
values first. Every cached value differs from what its formula gives.

Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS, SHEET_TYPES  # noqa: E402
from probe_package import content_types, output_directory, relationships, styles, write_package  # noqa: E402

STRINGS = ("zero", "one", "two", "three")
# Each file's cells of column A: (the cell's type, its formula or None, its stored value).
FILES = {
    "formula-string-value.xlsx": (
        ("s", '"x"&amp;"y"', "3"),  # a text formula cached as shared string 3
        ("s", "1+1", "2"),  # a number formula cached as shared string 2
        (
            "str",
            '"x"&amp;"y"',
            "stale",
        ),  # control: the form Excel writes for a text result, with a stale value
        ("s", None, "1"),  # control: a constant that is shared string 1
    ),
    "formula-string-value-out-of-range.xlsx": (
        ("s", '"q"', "9"),  # cached as shared string 9, and the table has four
        ("str", '"x"&amp;"y"', "stale"),  # control
    ),
}

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
    '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>'
    '<calcPr calcId="191029" calcMode="manual"/></workbook>'
)
SHARED_STRINGS = (
    f'{HEAD}<sst xmlns="{MAIN}" count="{len(STRINGS)}" uniqueCount="{len(STRINGS)}">'
    f"{''.join(f'<si><t>{text}</t></si>' for text in STRINGS)}</sst>"
)


def sheet(cells: tuple[tuple[str, str | None, str], ...]) -> str:
    rows = []
    for r, (kind, formula, value) in enumerate(cells, start=1):
        f = f"<f>{formula}</f>" if formula is not None else ""
        rows.append(f'<row r="{r}"><c r="A{r}" t="{kind}">{f}<v>{value}</v></c></row>')
    return f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData>{"".join(rows)}</sheetData></worksheet>'


def main() -> None:
    out = output_directory()
    types = content_types(
        1, styles=True, more=(("/xl/sharedStrings.xml", f"{SHEET_TYPES}.sharedStrings+xml"),)
    )
    workbook_parts = relationships(
        ("worksheet", "worksheets/sheet1.xml"),
        ("styles", "styles.xml"),
        ("sharedStrings", "sharedStrings.xml"),
    )
    for name, cells in FILES.items():
        write_package(
            out / name,
            (
                ("[Content_Types].xml", types),
                ("_rels/.rels", PACKAGE_RELATIONSHIPS),
                ("xl/workbook.xml", WORKBOOK),
                ("xl/_rels/workbook.xml.rels", workbook_parts),
                ("xl/styles.xml", styles()),
                ("xl/sharedStrings.xml", SHARED_STRINGS),
                ("xl/worksheets/sheet1.xml", sheet(cells)),
            ),
        )


if __name__ == "__main__":
    main()
