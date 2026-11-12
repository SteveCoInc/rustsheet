"""Hand-built probe packages for the load-refusal shapes (poc-cfix-loadref-cc).

Each package is a minimal SpreadsheetML package written here, with Application "RustSheet fixture"
(fixtures suite convention: placeholder cached values are never Excel evidence). Usage:
    python3 -I make_probes.py OUTDIR
"""

import sys
import zipfile
from pathlib import Path

MAIN = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_REL = "http://schemas.openxmlformats.org/package/2006/relationships"
CT_SHEET = "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"
CT_WB = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"
CT_STYLES = "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"
CT_COMMENTS = "application/vnd.openxmlformats-officedocument.spreadsheetml.comments+xml"
CT_TABLE = "application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml"

STYLES = (
    f'<styleSheet xmlns="{MAIN}"><fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>'
    '<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>'
    '<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>'
    '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
    '<cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>'
    '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>'
)

APP = (
    '<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">'
    "<Application>RustSheet fixture</Application></Properties>"
)


def cell(ref, value=None, formula=None, text=None):
    if text is not None:
        return f'<c r="{ref}" t="inlineStr"><is><t>{text}</t></is></c>'
    f = f"<f>{formula}</f>" if formula is not None else ""
    v = f"<v>{value}</v>" if value is not None else ""
    return f'<c r="{ref}">{f}{v}</c>'


def sheet_xml(rows, extra=""):
    body = "".join(f'<row r="{r}">{"".join(cells)}</row>' for r, cells in rows)
    return f'<worksheet xmlns="{MAIN}" xmlns:r="{REL}"><sheetData>{body}</sheetData>{extra}</worksheet>'


def package(path, sheets, wb_rels, parts, styles=True, overrides=()):
    """sheets: [(name, rid)]; wb_rels: [(rid, kind, target)]; parts: {name: xml}."""
    sheets_xml = "".join(
        f'<sheet name="{n}" sheetId="{i + 1}" r:id="{rid}"/>' for i, (n, rid) in enumerate(sheets)
    )
    workbook = f'<workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets>{sheets_xml}</sheets></workbook>'
    rels = list(wb_rels)
    if styles:
        rels.append(("rIdStyles", "styles", "styles.xml"))
    rels_xml = "".join(
        f'<Relationship Id="{rid}" Type="{REL}/{kind}" Target="{target}"/>' for rid, kind, target in rels
    )
    ct_over = [("/xl/workbook.xml", CT_WB)]
    ct_over += [
        ("/" + name, CT_SHEET) for name in parts if name.startswith("xl/worksheets/") and "_rels" not in name
    ]
    if styles:
        ct_over.append(("/xl/styles.xml", CT_STYLES))
    ct_over += list(overrides)
    content_types = (
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        + "".join(f'<Override PartName="{p}" ContentType="{c}"/>' for p, c in ct_over)
        + '<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>'
        "</Types>"
    )
    root_rels = (
        f'<Relationships xmlns="{PKG_REL}">'
        f'<Relationship Id="rId1" Type="{REL}/officeDocument" Target="xl/workbook.xml"/>'
        '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>'
        "</Relationships>"
    )
    entries = {
        "[Content_Types].xml": content_types,
        "_rels/.rels": root_rels,
        "docProps/app.xml": APP,
        "xl/workbook.xml": workbook,
        "xl/_rels/workbook.xml.rels": f'<Relationships xmlns="{PKG_REL}">{rels_xml}</Relationships>',
    }
    if styles:
        entries["xl/styles.xml"] = STYLES
    entries.update(parts)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, xml in entries.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 11, 12, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' + xml)


def calc_rows():
    """The questions a sheet asks about a sheet named Gone between Data and Calc."""
    return [
        (1, [cell("A1", 0, "_xlfn.SHEETS()")]),
        (2, [cell("A2", 0, '_xlfn.SHEET("Calc")')]),
        (3, [cell("A3", 0, "ISREF(Gone!A1)")]),
        (4, [cell("A4", 0, "Gone!A1+1")]),
        (5, [cell("A5", 0, "Data!A1*2")]),
        (6, [cell("A6", 0, "_xlfn.SHEETS(Data:Calc)")]),
        (7, [cell("A7", 0, "SUM(Data:Calc!B1)")]),
    ]


def main(out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    data = sheet_xml([(1, [cell("A1", 7), cell("B1", 1)])])
    calc = sheet_xml(calc_rows())
    # 1. No styles part at all; cells as inline strings and numbers.
    package(
        out / "package-no-styles-part.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml")],
        {
            "xl/worksheets/sheet1.xml": sheet_xml(
                [
                    (1, [cell("A1", text="abc"), cell("B1", 0, "LEN(A1)")]),
                    (2, [cell("A2", 5), cell("B2", 0, "A2*2")]),
                    (3, [cell("B3", 0, "ISTEXT(A1)")]),
                ]
            )
        },
        styles=False,
    )
    # 2. A sheet whose r:id names no relationship.
    package(
        out / "package-sheet-without-relationship.xlsx",
        [("Data", "rId1"), ("Gone", "rId9"), ("Calc", "rId2")],
        [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "worksheet", "worksheets/sheet2.xml")],
        {"xl/worksheets/sheet1.xml": data, "xl/worksheets/sheet2.xml": calc},
    )
    # 3. A sheet whose relationship names a part the package does not hold.
    package(
        out / "package-sheet-part-missing.xlsx",
        [("Data", "rId1"), ("Gone", "rId9"), ("Calc", "rId2")],
        [
            ("rId1", "worksheet", "worksheets/sheet1.xml"),
            ("rId2", "worksheet", "worksheets/sheet2.xml"),
            ("rId9", "worksheet", "worksheets/sheet9.xml"),
        ],
        {"xl/worksheets/sheet1.xml": data, "xl/worksheets/sheet2.xml": calc},
    )
    # 4. A sheet whose comments relationship names a part the package does not hold.
    package(
        out / "package-comments-part-missing.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml")],
        {
            "xl/worksheets/sheet1.xml": sheet_xml([(1, [cell("A1", 7), cell("B1", 0, "A1*2")])]),
            "xl/worksheets/_rels/sheet1.xml.rels": (
                f'<Relationships xmlns="{PKG_REL}"><Relationship Id="rId1" Type="{REL}/comments" '
                'Target="../comments1.xml"/></Relationships>'
            ),
        },
        overrides=[("/xl/comments1.xml", CT_COMMENTS)],
    )
    # 5. A sheet whose table relationship names a part the package does not hold.
    package(
        out / "package-table-part-missing.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml")],
        {
            "xl/worksheets/sheet1.xml": sheet_xml(
                [(1, [cell("A1", 7), cell("B1", 0, "A1*2")])],
                '<tableParts count="1"><tablePart r:id="rId1"/></tableParts>',
            ),
            "xl/worksheets/_rels/sheet1.xml.rels": (
                f'<Relationships xmlns="{PKG_REL}"><Relationship Id="rId1" Type="{REL}/table" '
                'Target="../tables/table1.xml"/></Relationships>'
            ),
        },
        overrides=[("/xl/tables/table1.xml", CT_TABLE)],
    )


if __name__ == "__main__":
    main(sys.argv[1])
