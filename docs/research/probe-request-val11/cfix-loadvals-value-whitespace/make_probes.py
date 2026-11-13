"""Hand-built probe packages: a cell's <v> with whitespace (or other padding) around its value
(poc-cfix-loadvals-cc, 2026-11-12).

Each package is a minimal SpreadsheetML package written here, with Application "RustSheet fixture"
(fixtures suite convention: placeholder cached values are never Excel evidence). One question group per
workbook, so a form Excel refuses takes only its own group down (a refusal is itself the answer).
Usage:
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
CT_SST = "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"

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


def raw(ref, v, t=None):
    """A constant cell whose <v> is written exactly as `v` (already XML-escaped)."""
    ta = f' t="{t}"' if t else ""
    return f'<c r="{ref}"{ta}><v>{v}</v></c>'


def formula(ref, f):
    return f'<c r="{ref}"><f>{f}</f><v>0</v></c>'


def sheet_xml(rows):
    body = "".join(f'<row r="{r}">{"".join(cells)}</row>' for r, cells in rows)
    return f'<worksheet xmlns="{MAIN}" xmlns:r="{REL}"><sheetData>{body}</sheetData></worksheet>'


def package(path, sheet, sst=None):
    workbook = (
        f'<workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets><sheet name="Probe" sheetId="1" r:id="rId1"/>'
        "</sheets></workbook>"
    )
    rels = [("rId1", "worksheet", "worksheets/sheet1.xml"), ("rId2", "styles", "styles.xml")]
    over = [
        ("/xl/workbook.xml", CT_WB),
        ("/xl/worksheets/sheet1.xml", CT_SHEET),
        ("/xl/styles.xml", CT_STYLES),
    ]
    if sst is not None:
        rels.append(("rId3", "sharedStrings", "sharedStrings.xml"))
        over.append(("/xl/sharedStrings.xml", CT_SST))
    rels_xml = "".join(f'<Relationship Id="{i}" Type="{REL}/{k}" Target="{t}"/>' for i, k, t in rels)
    content_types = (
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        + "".join(f'<Override PartName="{p}" ContentType="{c}"/>' for p, c in over)
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
        "xl/styles.xml": STYLES,
        "xl/worksheets/sheet1.xml": sheet,
    }
    if sst is not None:
        items = "".join(f"<si><t>{s}</t></si>" for s in sst)
        entries["xl/sharedStrings.xml"] = (
            f'<sst xmlns="{MAIN}" count="{len(sst)}" uniqueCount="{len(sst)}">{items}</sst>'
        )
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, xml in entries.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 11, 12, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' + xml)


def rows(values, readers, t=None):
    """Row n: A the stored form values[n-1], then one reader per column from B."""
    out = []
    for n, v in enumerate(values, start=1):
        cells = [raw(f"A{n}", v, t)]
        for col, reader in zip("BCDEFG", readers):
            cells.append(formula(f"{col}{n}", reader.format(a=f"A{n}")))
        out.append((n, cells))
    return out


NUMBER_READERS = ["ISNUMBER({a})", "{a}+1", "TYPE({a})", "ISBLANK({a})", '{a}&amp;"|"']


def main(out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    # 1. A number with XML whitespace around it; A4 the control.
    package(out / "value-ws-numbers.xlsx", sheet_xml(rows([" 9", "9 ", "&#9;9&#10;", "9"], NUMBER_READERS)))
    # 2. Other paddings: a no-break space (Unicode, not XML, whitespace), a leading +, a space inside, a
    #    whitespace-only value.
    package(
        out / "value-ws-number-forms.xlsx", sheet_xml(rows(["&#160;9", "+9", "9 0", " "], NUMBER_READERS))
    )
    # 3. t="b": padded 1, and the xsd:boolean spellings ECMA-376 does not give a cell; A4 the control.
    package(
        out / "value-ws-booleans.xlsx",
        sheet_xml(
            rows(
                [" 1 ", "true", "TRUE", "1"], ["ISLOGICAL({a})", "{a}=TRUE", "TYPE({a})", '{a}&amp;"|"'], "b"
            )
        ),
    )
    # 4. t="e": a padded error name; A2 the control.
    package(
        out / "value-ws-errors.xlsx",
        sheet_xml(rows([" #REF! ", "#REF!"], ["ERROR.TYPE({a})", "ISERROR({a})", "TYPE({a})"], "e")),
    )
    # 6. t="d": a padded ISO date; A2 the control. Its own workbook: Excel refuses a workbook whose t="d" text it
    #    cannot read (xlsx-storage.md), so a refusal here takes nothing else down.
    package(
        out / "value-ws-date.xlsx",
        sheet_xml(rows([" 2024-01-01 ", "2024-01-01"], ["ISNUMBER({a})", "{a}+1", "TYPE({a})"], "d")),
    )
    # 7. Whitespace-only values: <v> </v> as a number, a boolean and an error; one workbook per type, so a refusal
    #    names its type.
    for t, name in ((None, "n"), ("b", "b"), ("e", "e")):
        package(
            out / f"value-ws-only-{name}.xlsx",
            sheet_xml(rows([" "], ["ISBLANK({a})", "TYPE({a})", "ISNUMBER({a})", "ISERROR({a})", "{a}&amp;\"|\""], t)),
        )
    # 5. t="s": a padded shared-string index (T-4711's shapes); A5 the control. sst 0 "zero", 1 "one".
    package(
        out / "value-ws-shared-index.xlsx",
        sheet_xml(rows([" 1", "1 ", "+1", "&#160;1", "1"], ['{a}&amp;"|"', "ISTEXT({a})", "LEN({a})"], "s")),
        sst=["zero", "one"],
    )


if __name__ == "__main__":
    main(sys.argv[1])
