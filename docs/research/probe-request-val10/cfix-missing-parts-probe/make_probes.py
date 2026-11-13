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


def package(path, sheets, wb_rels, parts, styles=True, overrides=(), defined_names=""):
    """sheets: [(name, rid)]; wb_rels: [(rid, kind, target)]; parts: {name: xml}."""
    sheets_xml = "".join(
        f'<sheet name="{n}" sheetId="{i + 1}" r:id="{rid}"/>' for i, (n, rid) in enumerate(sheets)
    )
    workbook = f'<workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets>{sheets_xml}</sheets>{defined_names}</workbook>'
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


if __name__ == "__main__" and len(sys.argv) == 2:
    main(sys.argv[1])


def name_scoped_to_no_sheet(out):
    """A workbook of Data and Calc whose name `Lost` has localSheetId 5 (past the two sheets)."""
    out = Path(out)
    data = sheet_xml([(1, [cell("A1", 7)]), (2, [cell("A2", 8)]), (3, [cell("A3", 9)])])
    calc = sheet_xml([
        (1, [cell("A1", 0, "Lost")]),
        (2, [cell("A2", 0, "ISREF(Lost)")]),
        (3, [cell("A3", 0, "Global")]),
        (4, [cell("A4", 0, "Data!Kept")]),
        (5, [cell("A5", 0, "Data!A1*2")]),
    ])
    package(
        out / "package-name-scoped-to-no-sheet.xlsx",
        [("Data", "rId1"), ("Calc", "rId2")],
        [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "worksheet", "worksheets/sheet2.xml")],
        {"xl/worksheets/sheet1.xml": data, "xl/worksheets/sheet2.xml": calc},
        defined_names=(
            '<definedNames><definedName name="Lost" localSheetId="5">Data!$A$1</definedName>'
            '<definedName name="Kept" localSheetId="0">Data!$A$2</definedName>'
            '<definedName name="Global">Data!$A$3</definedName></definedNames>'
        ),
    )


if __name__ == "__main__" and len(sys.argv) > 2 and sys.argv[2] == "names":
    name_scoped_to_no_sheet(sys.argv[1])


STYLES_PERCENT = STYLES.replace(
    '<cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>',
    '<cellXfs count="2"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
    '<xf numFmtId="9" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/></cellXfs>',
)


def percent_sheet():
    """A1 = 0.5 with style 1 (percent, if the styles part read is STYLES_PERCENT); B1 asks its format."""
    return (
        f'<worksheet xmlns="{MAIN}" xmlns:r="{REL}"><sheetData><row r="1">'
        '<c r="A1" s="1"><v>0.5</v></c><c r="B1" t="str"><f>CELL("format",A1)</f><v>0</v></c>'
        "</row></sheetData></worksheet>"
    )


def round2(out):
    """The shapes the CRs of 2026-11-12 load or refuse after review (see README.md, 'Added 13h')."""
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    data = sheet_xml([(1, [cell("A1", 7), cell("B1", 1)])])
    calc = sheet_xml(calc_rows())
    # 6. A very hidden sheet listed with an empty r:id (the shape the importer loads empty).
    workbook_sheets = [("Data", "rId1"), ("Gone", ""), ("Calc", "rId2")]
    path = out / "package-sheet-empty-rid.xlsx"
    package(
        path,
        workbook_sheets,
        [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "worksheet", "worksheets/sheet2.xml")],
        {"xl/worksheets/sheet1.xml": data, "xl/worksheets/sheet2.xml": calc},
    )
    rewrite(path, "xl/workbook.xml", lambda x: x.replace('name="Gone" sheetId="2" r:id=""', 'name="Gone" sheetId="2" state="veryHidden" r:id=""'))
    # 7. The styles part at another name, named by the workbook's relationship.
    package(
        out / "package-styles-relocated.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "styles", "formatting/custom.xml")],
        {"xl/worksheets/sheet1.xml": percent_sheet(), "xl/formatting/custom.xml": STYLES_PERCENT},
        styles=False,
        overrides=[("/xl/formatting/custom.xml", CT_STYLES)],
    )
    # 8. A styles relationship to a part the package lacks, beside a valid xl/styles.xml it does not name.
    package(
        out / "package-styles-dangling-relationship.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "styles", "formatting/custom.xml")],
        {"xl/worksheets/sheet1.xml": percent_sheet(), "xl/styles.xml": STYLES_PERCENT},
        styles=False,
        overrides=[("/xl/styles.xml", CT_STYLES)],
    )
    # 9. A row and a column whose style names no cell format (cellXfs has 1).
    package(
        out / "package-row-style-past-cellxfs.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml")],
        {
            "xl/worksheets/sheet1.xml": (
                f'<worksheet xmlns="{MAIN}" xmlns:r="{REL}"><cols><col min="3" max="3" width="9" style="7"/></cols>'
                '<sheetData><row r="1" s="5" customFormat="1"><c r="A1"><v>0.5</v></c>'
                '<c r="B1" t="str"><f>CELL("format",A1)</f><v>0</v></c></row>'
                '<row r="2"><c r="C2"><v>0.25</v></c><c r="D2" t="str"><f>CELL("format",C2)</f><v>0</v></c></row>'
                "</sheetData></worksheet>"
            )
        },
    )


def rewrite(path, name, edit):
    """Rewrites one entry of a zip written by package(), keeping the others and their order."""
    with zipfile.ZipFile(path) as z:
        items = [(i, z.read(i.filename)) for i in z.infolist()]
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for info, data in items:
            if info.filename == name:
                data = edit(data.decode()).encode()
            z.writestr(info, data)


if __name__ == "__main__" and len(sys.argv) > 2 and sys.argv[2] == "round2":
    round2(sys.argv[1])


def round3(out):
    """The row shape 9 of the 10 corpus files hold, in its own workbook (CR-322034747 round 2, reviewer 1 N3)."""
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    package(
        out / "package-row-style-negative.xlsx",
        [("Data", "rId1")],
        [("rId1", "worksheet", "worksheets/sheet1.xml")],
        {
            "xl/worksheets/sheet1.xml": (
                f'<worksheet xmlns="{MAIN}" xmlns:r="{REL}"><sheetData>'
                '<row r="1" ht="30" customHeight="1" s="-2147483648" customFormat="1"><c r="A1"><v>0.5</v></c>'
                '<c r="B1" t="str"><f>CELL("format",A1)</f><v>0</v></c></row>'
                "</sheetData></worksheet>"
            )
        },
    )


if __name__ == "__main__" and len(sys.argv) > 2 and sys.argv[2] == "round3":
    round3(sys.argv[1])
