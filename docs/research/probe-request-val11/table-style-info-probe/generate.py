#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md): one sheet with one table of two columns, and a
formula that reads the table. The files differ only in the table's `<tableStyleInfo>` and, for two of them,
in whether the styles part defines the style the table names.

Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS, SHEET_TYPES  # noqa: E402
from probe_package import content_types, output_directory, relationships, styles  # noqa: E402
from probe_package import workbook_relationships, write_package  # noqa: E402

FLAGS = ("showFirstColumn", "showLastColumn", "showRowStripes", "showColumnStripes")
# The flags as Excel writes them for a new table, and their opposites.
USUAL = {"showFirstColumn": "0", "showLastColumn": "0", "showRowStripes": "1", "showColumnStripes": "0"}
OPPOSITE = {flag: "1" if value == "0" else "0" for flag, value in USUAL.items()}
BUILT_IN, CUSTOM = "TableStyleMedium2", "ProbeStyle"


def style_info(name: str | None, flags: dict[str, str]) -> str:
    """A `<tableStyleInfo>` with the attributes given, in the schema's order."""
    named = f' name="{name}"' if name is not None else ""
    return f"<tableStyleInfo{named}{''.join(f' {flag}="{flags[flag]}"' for flag in FLAGS if flag in flags)}/>"


def without(flag: str) -> dict[str, str]:
    return {other: value for other, value in USUAL.items() if other != flag}


# Each file: the table's `<tableStyleInfo>` ("" for none), and whether the styles part defines `ProbeStyle`.
FILES = {
    "table-control.xlsx": (style_info(BUILT_IN, USUAL), False),
    "table-control-opposite.xlsx": (style_info(BUILT_IN, OPPOSITE), False),
    "table-no-first-column.xlsx": (style_info(BUILT_IN, without("showFirstColumn")), False),
    "table-no-last-column.xlsx": (style_info(BUILT_IN, without("showLastColumn")), False),
    "table-no-row-stripes.xlsx": (style_info(BUILT_IN, without("showRowStripes")), False),
    "table-no-column-stripes.xlsx": (style_info(BUILT_IN, without("showColumnStripes")), False),
    "table-no-flags.xlsx": (style_info(BUILT_IN, {}), False),
    "table-no-style-info.xlsx": ("", False),
    "table-style-undefined.xlsx": (style_info(CUSTOM, USUAL), False),
    "table-style-defined.xlsx": (style_info(CUSTOM, USUAL), True),
}

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
    '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>'
    '<calcPr calcId="191029" fullCalcOnLoad="1"/></workbook>'
)
# A1:B3 is the table: a header row and two rows. D1 reads it, so its value says the table is one.
SHEET = (
    f'{HEAD}<worksheet xmlns="{MAIN}" xmlns:r="{OFFICE}"><dimension ref="A1:D3"/><sheetData>'
    '<row r="1"><c r="A1" t="inlineStr"><is><t>k</t></is></c><c r="B1" t="inlineStr"><is><t>n</t></is></c>'
    '<c r="D1"><f>SUM(Probe[n])</f></c></row>'
    '<row r="2"><c r="A2" t="inlineStr"><is><t>a</t></is></c><c r="B2"><v>10</v></c></row>'
    '<row r="3"><c r="A3" t="inlineStr"><is><t>b</t></is></c><c r="B3"><v>20</v></c></row>'
    '</sheetData><tableParts count="1"><tablePart r:id="rId1"/></tableParts></worksheet>'
)
# The style `ProbeStyle`: a whole-table element with a bold font.
TABLE_STYLES = (
    '<dxfs count="1"><dxf><font><b/></font></dxf></dxfs>'
    f'<tableStyles count="1" defaultTableStyle="{BUILT_IN}" defaultPivotStyle="PivotStyleLight16">'
    f'<tableStyle name="{CUSTOM}" pivot="0" count="1"><tableStyleElement type="wholeTable" dxfId="0"/>'
    "</tableStyle></tableStyles>"
)


def table(info: str) -> str:
    return (
        f'{HEAD}<table xmlns="{MAIN}" id="1" name="Probe" displayName="Probe" ref="A1:B3" totalsRowShown="0">'
        '<autoFilter ref="A1:B3"/><tableColumns count="2"><tableColumn id="1" name="k"/>'
        f'<tableColumn id="2" name="n"/></tableColumns>{info}</table>'
    )


def main() -> None:
    out = output_directory()
    types = content_types(1, styles=True, more=(("/xl/tables/table1.xml", f"{SHEET_TYPES}.table+xml"),))
    for name, (info, defined) in FILES.items():
        style_part = styles()
        if defined:
            # `dxfs` and `tableStyles` follow `cellStyles` in the styles part (CT_Stylesheet).
            style_part = style_part.replace("</cellStyles>", f"</cellStyles>{TABLE_STYLES}")
        write_package(
            out / name,
            (
                ("[Content_Types].xml", types),
                ("_rels/.rels", PACKAGE_RELATIONSHIPS),
                ("xl/workbook.xml", WORKBOOK),
                ("xl/_rels/workbook.xml.rels", workbook_relationships(1, styles=True)),
                ("xl/styles.xml", style_part),
                ("xl/worksheets/sheet1.xml", SHEET),
                ("xl/worksheets/_rels/sheet1.xml.rels", relationships(("table", "../tables/table1.xml"))),
                ("xl/tables/table1.xml", table(info)),
            ),
        )


if __name__ == "__main__":
    main()
