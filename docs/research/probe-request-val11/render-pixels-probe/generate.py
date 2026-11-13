#!/usr/bin/env python3
"""Writes the workbook of the probe request (README.md): `render-pixels.xlsx`.

Sheet `Borders` has one boxed cell for each of five border styles (thin, medium, thick, double, dashed), a
thin line under B4:J4, and the default gridlines everywhere else. Sheet `Text` has five lines of black
Calibri 11 in column A, the second on a yellow fill. Nothing else is formatted, so that a screenshot shows
Excel's own drawing of a border, a gridline and a glyph.

Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, workbook_relationships, write_package  # noqa: E402

# The boxed cells of `Borders`, in the order of the styles part's borders 1 to 5.
BOXES = (("B2", "thin"), ("D2", "medium"), ("F2", "thick"), ("H2", "double"), ("J2", "dashed"))
# The lines of `Text`: (cell, text, cell format). Format 7 is the yellow fill.
LINES = (
    ("A1", "Hamburgefonstiv 0123456789", 0),
    ("A2", "Hamburgefonstiv 0123456789", 7),
    ("A3", "illiill WAVE To AV fi fl", 0),
    ("A4", "abcdefghijklmnopqrstuvwxyz", 0),
    ("A5", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", 0),
)

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}"><sheets>'
    '<sheet name="Borders" sheetId="1" r:id="rId1"/><sheet name="Text" sheetId="2" r:id="rId2"/>'
    "</sheets></workbook>"
)


def box(style: str) -> str:
    side = f'style="{style}"><color auto="1"/>'
    return f"<border><left {side}</left><right {side}</right><top {side}</top><bottom {side}</bottom><diagonal/></border>"


def styles() -> str:
    """One font, three fills, seven borders; cell formats 1 to 5 box a cell, 6 underlines it, 7 fills it."""
    borders = (
        "<border><left/><right/><top/><bottom/><diagonal/></border>"
        + "".join(box(style) for _, style in BOXES)
        + '<border><left/><right/><top/><bottom style="thin"><color auto="1"/></bottom><diagonal/></border>'
    )
    formats = '<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>' + "".join(
        f'<xf numFmtId="0" fontId="0" fillId="0" borderId="{n}" xfId="0" applyBorder="1"/>' for n in range(1, 7)
    )
    formats += '<xf numFmtId="0" fontId="0" fillId="2" borderId="0" xfId="0" applyFill="1"/>'
    return (
        f'{HEAD}<styleSheet xmlns="{MAIN}">'
        '<fonts count="1"><font><sz val="11"/><color rgb="FF000000"/><name val="Calibri"/></font></fonts>'
        '<fills count="3"><fill><patternFill patternType="none"/></fill>'
        '<fill><patternFill patternType="gray125"/></fill>'
        '<fill><patternFill patternType="solid"><fgColor rgb="FFFFFF00"/></patternFill></fill></fills>'
        f'<borders count="7">{borders}</borders>'
        '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
        f'<cellXfs count="8">{formats}</cellXfs>'
        '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>'
        "</styleSheet>"
    )


def text(address: str, value: str, fmt: int = 0) -> str:
    return f'<c r="{address}" s="{fmt}" t="inlineStr"><is><t>{value}</t></is></c>'


def borders_sheet() -> str:
    names = "".join(text(f"{address[0]}1", style) for address, style in BOXES)
    boxed = "".join(f'<c r="{address}" s="{n}"/>' for n, (address, _) in enumerate(BOXES, 1))
    line = "".join(f'<c r="{column}4" s="6"/>' for column in "BCDEFGHIJ")
    rows = f'<row r="1">{names}</row><row r="2">{boxed}</row><row r="4">{line}</row>'
    return f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData>{rows}</sheetData></worksheet>'


def text_sheet() -> str:
    rows = "".join(f'<row r="{address[1:]}">{text(address, value, fmt)}</row>' for address, value, fmt in LINES)
    return (
        f'{HEAD}<worksheet xmlns="{MAIN}"><cols><col min="1" max="1" width="60" customWidth="1"/></cols>'
        f"<sheetData>{rows}</sheetData></worksheet>"
    )


def main() -> None:
    out = output_directory()
    write_package(
        out / "render-pixels.xlsx",
        (
            ("[Content_Types].xml", content_types(2, styles=True)),
            ("_rels/.rels", PACKAGE_RELATIONSHIPS),
            ("xl/workbook.xml", WORKBOOK),
            ("xl/_rels/workbook.xml.rels", workbook_relationships(2, styles=True)),
            ("xl/styles.xml", styles()),
            ("xl/worksheets/sheet1.xml", borders_sheet()),
            ("xl/worksheets/sheet2.xml", text_sheet()),
        ),
    )


if __name__ == "__main__":
    main()
