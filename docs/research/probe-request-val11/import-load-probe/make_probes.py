"""Hand-built probe packages for the import's remaining load work (poc-cfix-uiperf-cc, 2026-11-12).

Each package is a minimal SpreadsheetML package written here, with Application "RustSheet fixture" (cached values
are placeholders, never Excel evidence). Usage:
    python3 -I make_probes.py OUTDIR
"""

import sys
import zipfile
from pathlib import Path

MAIN = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_REL = "http://schemas.openxmlformats.org/package/2006/relationships"

CONTENT_TYPES = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
    '<Default Extension="xml" ContentType="application/xml"/>'
    '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>'
    '<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>'
    '<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>'
    '<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>'
    "</Types>"
)
ROOT_RELS = (
    f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="{PKG_REL}">'
    '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>'
    '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>'
    "</Relationships>"
)
APP = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">'
    "<Application>RustSheet fixture</Application></Properties>"
)
WORKBOOK = (
    f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="{MAIN}" xmlns:r="{REL}">'
    '<sheets><sheet name="Data" sheetId="1" r:id="rId1"/></sheets></workbook>'
)
WORKBOOK_RELS = (
    f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="{PKG_REL}">'
    '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>'
    '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
    "</Relationships>"
)


def styles(xfs: str) -> str:
    """A styles part whose cellXfs are `xfs` (style 0 first, General with no wrap)."""
    n = xfs.count("<xf ")
    return (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet xmlns="{MAIN}">'
        '<fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>'
        '<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>'
        '<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>'
        '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
        f'<cellXfs count="{n}">{xfs}</cellXfs>'
        '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>'
    )


GENERAL = '<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
WRAP = '<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0" applyAlignment="1"><alignment wrapText="1"/></xf>'
TWO_DP = '<xf numFmtId="2" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/>'
PERCENT = '<xf numFmtId="9" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/>'
LONG = "a line of text long enough to wrap twice or more in a column of the default width " * 3


def sheet(body: str, fmt: str = "", cols: str = "", merges: str = "") -> str:
    return (
        f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="{MAIN}" xmlns:r="{REL}">'
        f"{fmt}{cols}<sheetData>{body}</sheetData>{merges}</worksheet>"
    )


def inline(ref: str, text: str, s: int = 0) -> str:
    return f'<c r="{ref}" s="{s}" t="inlineStr"><is><t>{text}</t></is></c>'


PROBES = {
    # P-1a: every row a custom 30 pt by the sheet's default; A1 wrapped long text, its row not listed.
    "sizes-default-custom.xlsx": (
        sheet(f'<row r="2">{inline("A2", LONG, 1)}</row>', fmt='<sheetFormatPr customHeight="1" defaultRowHeight="30"/>'),
        styles(GENERAL + WRAP),
    ),
    # P-1b: the same default height, not custom.
    "sizes-default-plain.xlsx": (
        sheet(f'<row r="2">{inline("A2", LONG, 1)}</row>', fmt='<sheetFormatPr defaultRowHeight="30"/>'),
        styles(GENERAL + WRAP),
    ),
    # P-2: two merges that overlap at B2.
    "merges-overlap.xlsx": (
        sheet(
            f'<row r="1">{inline("A1", "first")}</row><row r="2">{inline("B2", "shared")}</row>',
            merges='<mergeCells count="2"><mergeCell ref="A1:B2"/><mergeCell ref="B2:C3"/></mergeCells>',
        ),
        styles(GENERAL),
    ),
    # P-3: column B styled 0.00, row 3 styled 0%, B5 a cell of its own with General, B6 and D3 no cell.
    "numfmt-col-row.xlsx": (
        sheet(
            '<row r="3" s="2" customFormat="1"/><row r="5"><c r="B5" s="0"/></row>',
            cols='<cols><col min="2" max="2" width="9.140625" style="1" customWidth="1"/></cols>',
        ),
        styles(GENERAL + TWO_DP + PERCENT),
    ),
}


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for name, (sheet_xml, styles_xml) in PROBES.items():
        with zipfile.ZipFile(out / name, "w", zipfile.ZIP_DEFLATED) as z:
            z.writestr("[Content_Types].xml", CONTENT_TYPES)
            z.writestr("_rels/.rels", ROOT_RELS)
            z.writestr("docProps/app.xml", APP)
            z.writestr("xl/workbook.xml", WORKBOOK)
            z.writestr("xl/_rels/workbook.xml.rels", WORKBOOK_RELS)
            z.writestr("xl/worksheets/sheet1.xml", sheet_xml)
            z.writestr("xl/styles.xml", styles_xml)
        print(out / name)


if __name__ == "__main__":
    main()
