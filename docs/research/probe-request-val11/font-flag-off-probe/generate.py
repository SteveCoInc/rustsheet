#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md): cells of column A, each with a font of its own that
states bold, italic, strikethrough or underline in one form, and the cell's text naming the form.

`font-flags.xlsx` holds the forms Excel writes or the schema allows. Each other file holds one `val` Excel
does not write, in A1, beside a plain bold cell in A2, so that a file Excel refuses says which. Three of them
are no `xsd:boolean`; the padded one is, since the type collapses whitespace.

Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""

import sys
from pathlib import Path
from xml.sax.saxutils import escape

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, workbook_relationships, write_package  # noqa: E402

# The fonts of `font-flags.xlsx`, row by row: what the font element holds before its size and name.
ALLOWED = (
    "",
    "<b/>",
    '<b val="1"/>',
    '<b val="true"/>',
    '<b val="0"/>',
    '<b val="false"/>',
    "<i/>",
    '<i val="false"/>',
    "<strike/>",
    '<strike val="0"/>',
    "<u/>",
    '<u val="none"/>',
    '<u val="single"/>',
)
# The other files: a `val` Excel does not write (the importer reads the first two as off and the last two
# as on), then the control.
NOT_ALLOWED = {
    "font-flag-upper-case.xlsx": '<b val="FALSE"/>',
    "font-flag-padded.xlsx": '<b val=" 0 "/>',
    "font-flag-word.xlsx": '<b val="off"/>',
    "font-flag-empty.xlsx": '<b val=""/>',
}

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
    '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>'
    '<calcPr calcId="191029" fullCalcOnLoad="1"/></workbook>'
)


def styles(fonts: tuple[str, ...]) -> str:
    """A styles part whose cell format i + 1 has the font `fonts[i]`; cell format 0 and font 0 are the default."""
    listed = "".join(f'<font>{flags}<sz val="11"/><name val="Calibri"/></font>' for flags in ("", *fonts))
    formats = "".join(
        f'<xf numFmtId="0" fontId="{i}" fillId="0" borderId="0" xfId="0" applyFont="1"/>'
        for i in range(1, len(fonts) + 1)
    )
    return (
        f'{HEAD}<styleSheet xmlns="{MAIN}"><fonts count="{len(fonts) + 1}">{listed}</fonts>'
        '<fills count="2"><fill><patternFill patternType="none"/></fill>'
        '<fill><patternFill patternType="gray125"/></fill></fills>'
        '<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>'
        '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
        f'<cellXfs count="{len(fonts) + 1}"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
        f"{formats}</cellXfs>"
        '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>'
    )


def sheet(fonts: tuple[str, ...]) -> str:
    """Column A: row i has cell format i and, as its text, the form of its font ("plain" for none). B1 counts
    them, so that its value says Excel calculated the workbook."""
    rows = []
    for i, flags in enumerate(fonts, start=1):
        text = escape(flags.strip("</>")) or "plain"
        count = f'<c r="B1"><f>COUNTA(A1:A{len(fonts)})</f></c>' if i == 1 else ""
        rows.append(f'<row r="{i}"><c r="A{i}" s="{i}" t="inlineStr"><is><t>{text}</t></is></c>{count}</row>')
    return f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData>{"".join(rows)}</sheetData></worksheet>'


def main() -> None:
    out = output_directory()
    for name, fonts in {
        "font-flags.xlsx": ALLOWED,
        **{n: (f, "<b/>") for n, f in NOT_ALLOWED.items()},
    }.items():
        write_package(
            out / name,
            (
                ("[Content_Types].xml", content_types(1, styles=True)),
                ("_rels/.rels", PACKAGE_RELATIONSHIPS),
                ("xl/workbook.xml", WORKBOOK),
                ("xl/_rels/workbook.xml.rels", workbook_relationships(1, styles=True)),
                ("xl/styles.xml", styles(fonts)),
                ("xl/worksheets/sheet1.xml", sheet(fonts)),
            ),
        )


if __name__ == "__main__":
    main()
