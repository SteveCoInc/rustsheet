#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md): one well-formed control, and seven copies of it that
each hold one undeclared entity reference, `&nbsp;`, in one part: a worksheet's text, a worksheet's attribute,
the shared strings, the styles, the theme, the application properties and the core properties.

The theme and the application properties are those of an Excel-written workbook of the repository, so that the
control is a package Excel opens. Where a case asks what Excel kept of a part, the control holds a value that
is no default of Excel's (a row height, a font, a colour scheme), so that the answer names its source.
Standard library only, and the same bytes at every run.
Usage: python3 -I generate.py <output directory>
"""
import sys
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS, SHEET_TYPES  # noqa: E402
from probe_package import content_types, output_directory, styles, workbook_relationships  # noqa: E402
from probe_package import write_package  # noqa: E402

REPOSITORY = Path(__file__).resolve().parents[3]
SOURCE = REPOSITORY / "crates/rustsheet-xlsx/tests/calc_tests/DATE_AND_TIME/DATEVALUE.xlsx"
THEME, APP, CORE = "xl/theme/theme1.xml", "docProps/app.xml", "docProps/core.xml"
SHEET, STRINGS = "xl/worksheets/sheet1.xml", "xl/sharedStrings.xml"
PACKAGE_TYPES = "application/vnd.openxmlformats-package"
CORE_RELATIONSHIP = "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"

WORKBOOK = (
    f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
    '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>'
    '<calcPr calcId="191029" fullCalcOnLoad="1"/></workbook>'
)
# A1 is text in the sheet and A2 text in the shared strings; B1 is a number and C1 the formula whose value says
# that Excel calculated the workbook. D and E read each text: its length, and the code of its second character
# (160 for U+00A0, 32 for a space, the letter's if the entity was dropped), which a person can read on screen.
# Row 1 has a height of its own, 30 points: Excel gives 15 to a row whose file sets none (render.md).
SHEET_XML = (
    f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData><row r="1" ht="30" customHeight="1">'
    '<c r="A1" t="inlineStr"><is><t>a b</t></is></c><c r="B1"><v>1</v></c>'
    '<c r="C1"><f>B1+1</f></c><c r="D1"><f>LEN(A1)</f></c>'
    '<c r="E1"><f>_xlfn.UNICODE(MID(A1,2,1))</f></c></row>'
    '<row r="2"><c r="A2" t="s"><v>0</v></c><c r="D2"><f>LEN(A2)</f></c>'
    '<c r="E2"><f>_xlfn.UNICODE(MID(A2,2,1))</f></c></row>'
    "</sheetData></worksheet>"
)
STRINGS_XML = f'{HEAD}<sst xmlns="{MAIN}" count="1" uniqueCount="1"><si><t>c d</t></si></sst>'
# Written here, with a title alone: an Excel-written core part names its author.
CORE_XML = (
    f'{HEAD}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" '
    'xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Probe title</dc:title></cp:coreProperties>'
)

# Each case: the part it changes, the text of the control, and the same text with the entity in it.
CASES = {
    "entity-sheet-text.xlsx": (SHEET, "<t>a b</t>", "<t>a&nbsp;b</t>"),
    "entity-sheet-attribute.xlsx": (SHEET, 'ht="30"', 'ht="30&nbsp;"'),
    "entity-shared-strings.xlsx": (STRINGS, "<t>c d</t>", "<t>c&nbsp;d</t>"),
    "entity-styles.xlsx": ("xl/styles.xml", 'val="Courier New"', 'val="Courier New&nbsp;"'),
    "entity-theme.xlsx": (THEME, '<a:clrScheme name="Probe">', '<a:clrScheme name="Probe&nbsp;">'),
    "entity-app.xlsx": (APP, "<Application>Microsoft Excel", "<Application>Microsoft&nbsp;Excel"),
    "entity-core.xlsx": (CORE, "<dc:title>Probe title", "<dc:title>Probe&nbsp;title"),
}


def marked(text: str, default: str, marker: str) -> str:
    """`text` with a value Excel gives by default replaced by one it never does, so that an answer names its source."""
    if text.count(default) != 1:
        sys.exit(f"the control does not hold {default!r} exactly once")
    return text.replace(default, marker)


def control() -> dict[str, str]:
    """The parts of the control, by name, in the order they are written."""
    with zipfile.ZipFile(SOURCE) as source:
        theme, app = source.read(THEME).decode("utf-8"), source.read(APP).decode("utf-8")
    # The shared skeleton names the application properties part already; here its text is Excel's own.
    types = content_types(1, styles=True).replace(
        "</Types>",
        '<Override PartName="/xl/theme/theme1.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>'
        f'<Override PartName="/{STRINGS}" ContentType="{SHEET_TYPES}.sharedStrings+xml"/>'
        f'<Override PartName="/{CORE}" ContentType="{PACKAGE_TYPES}.core-properties+xml"/></Types>',
    )
    package = PACKAGE_RELATIONSHIPS.replace(
        "</Relationships>", f'<Relationship Id="rId3" Type="{CORE_RELATIONSHIP}" Target="{CORE}"/></Relationships>'
    )
    book = workbook_relationships(1, styles=True).replace(
        "</Relationships>",
        f'<Relationship Id="rId3" Type="{OFFICE}/theme" Target="theme/theme1.xml"/>'
        f'<Relationship Id="rId4" Type="{OFFICE}/sharedStrings" Target="sharedStrings.xml"/></Relationships>',
    )
    # The font, the scheme's name and its first accent are not Excel's defaults (Calibri 11, Office, 5B9BD5).
    font = marked(styles(), '<sz val="11"/><name val="Calibri"/>', '<sz val="13"/><name val="Courier New"/>')
    scheme = marked(theme, '<a:clrScheme name="Office">', '<a:clrScheme name="Probe">')
    accent = '<a:accent1><a:srgbClr val="{}"/></a:accent1>'
    return {
        "[Content_Types].xml": types,
        "_rels/.rels": package,
        "xl/workbook.xml": WORKBOOK,
        "xl/_rels/workbook.xml.rels": book,
        "xl/styles.xml": font,
        THEME: marked(scheme, accent.format("5B9BD5"), accent.format("123456")),
        APP: app,
        CORE: CORE_XML,
        STRINGS: STRINGS_XML,
        SHEET: SHEET_XML,
    }


def main() -> None:
    out = output_directory()
    parts = control()
    write_package(out / "entity-control.xlsx", tuple(parts.items()))
    for name, (part, plain, with_entity) in CASES.items():
        if parts[part].count(plain) != 1:
            sys.exit(f"{part} does not hold {plain!r} exactly once")
        changed = {**parts, part: parts[part].replace(plain, with_entity)}
        write_package(out / name, tuple(changed.items()))


if __name__ == "__main__":
    main()
