"""Build merged-covered-ops.xlsx: the start workbook of the merged-range operation probes (stdlib only).

Usage: python3 -I build_merged_fixture.py OUT.xlsx

Sheets (all covered content is stored in the file, as fixtures/merged-covered-values does; Excel's UI
cannot create it, Format Painter or a file can):
  M (ops)      B2:C3 merged. B2 = 5 (anchor), C2 = "x" (covered text), B3 = =B2+1 (covered formula,
               cached 6), C3 empty. Same shape as the engine's the covered-cells spec (excel-parity-findings-2026-11-07.md section 1) unit tests.
  F (autofill) B2 = 1, B3 = 2, B4:C5 merged with B4 = 3 (anchor), C4 = 100, B5 = 200 (covered),
               C2 = 10, C3 = 20 (plain).
  N (Ctrl+arrow, row 2, 8 and column A/H blocks)
               row 2:  A2 = 1, B2:D2 merged, anchor B2 EMPTY, C2 = "h" (covered), F2 = 7
               row 8:  A8 = 1, B8:D8 merged, B8 = "a", C8 = "h" (covered), F8 = 7
               col H:  H1 = 1, H2:H4 merged, anchor H2 EMPTY, H3 = "h" (covered), H6 = 7
  W (API write) B2:C2 merged, B2 = 1, C2 = 2 (covered).
"""
from __future__ import annotations

import sys
import zipfile
from xml.sax.saxutils import escape

NS = 'xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"'
RNS = 'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"'

# sheet -> (cells {ref: value | ("f", formula, cached)}, merges)
SHEETS = {
    "M": ({"B2": 5, "C2": "x", "B3": ("f", "B2+1", 6)}, ["B2:C3"]),
    "F": ({"B2": 1, "B3": 2, "C2": 10, "C3": 20, "B4": 3, "C4": 100, "B5": 200}, ["B4:C5"]),
    "N": ({"A2": 1, "C2": "h", "F2": 7, "A8": 1, "B8": "a", "C8": "h", "F8": 7, "H1": 1, "H3": "h", "H6": 7},
          ["B2:D2", "B8:D8", "H2:H4"]),
    "W": ({"B2": 1, "C2": 2}, ["B2:C2"]),
}


def split(ref: str) -> tuple[str, int]:
    i = next(i for i, ch in enumerate(ref) if ch.isdigit())
    return ref[:i], int(ref[i:])


def colnum(c: str) -> int:
    n = 0
    for ch in c:
        n = n * 26 + ord(ch) - 64
    return n


def sheet_xml(cells: dict, merges: list[str]) -> str:
    rows: dict[int, list[tuple[str, object]]] = {}
    for ref, v in cells.items():
        rows.setdefault(split(ref)[1], []).append((ref, v))
    out = [f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet {NS}><sheetData>']
    for r in sorted(rows):
        out.append(f'<row r="{r}">')
        for ref, v in sorted(rows[r], key=lambda t: colnum(split(t[0])[0])):
            if isinstance(v, tuple):
                out.append(f'<c r="{ref}"><f>{escape(v[1])}</f><v>{v[2]}</v></c>')
            elif isinstance(v, str):
                out.append(f'<c r="{ref}" t="inlineStr"><is><t>{escape(v)}</t></is></c>')
            else:
                out.append(f'<c r="{ref}"><v>{v}</v></c>')
        out.append("</row>")
    out.append("</sheetData>")
    if merges:
        out.append(f'<mergeCells count="{len(merges)}">' + "".join(f'<mergeCell ref="{m}"/>' for m in merges)
                   + "</mergeCells>")
    out.append("</worksheet>")
    return "".join(out)


APP = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/'
       'officeDocument/2006/extended-properties"><Application>RustSheet probe fixture</Application></Properties>')
STYLES = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet ' + NS + '><fonts count="1"><font>'
          '<sz val="11"/><name val="Calibri"/></font></fonts><fills count="2"><fill><patternFill patternType="none"/>'
          '</fill><fill><patternFill patternType="gray125"/></fill></fills><borders count="1"><border><left/><right/>'
          '<top/><bottom/><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" '
          'borderId="0"/></cellStyleXfs><cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
          '</cellXfs><cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>')


def build(path: str) -> None:
    names = list(SHEETS)
    ct = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
          '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
          '<Default Extension="xml" ContentType="application/xml"/>'
          '<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.'
          'extended-properties+xml"/>'
          '<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.'
          'spreadsheetml.styles+xml"/>'
          '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.'
          'spreadsheetml.sheet.main+xml"/>'
          + "".join(f'<Override PartName="/xl/worksheets/sheet{i + 1}.xml" ContentType="application/'
                    f'vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>' for i in range(len(names)))
          + "</Types>")
    rels = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/'
            'officeDocument" Target="xl/workbook.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/'
            'officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>')
    wb = (f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook {NS} {RNS}><sheets>'
          + "".join(f'<sheet name="{n}" sheetId="{i + 1}" r:id="rId{i + 1}"/>' for i, n in enumerate(names))
          + '</sheets><calcPr fullCalcOnLoad="1"/></workbook>')
    wbrels = ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
              '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
              + "".join(f'<Relationship Id="rId{i + 1}" Type="http://schemas.openxmlformats.org/officeDocument/'
                        f'2006/relationships/worksheet" Target="worksheets/sheet{i + 1}.xml"/>'
                        for i in range(len(names)))
              + f'<Relationship Id="rId{len(names) + 1}" Type="http://schemas.openxmlformats.org/officeDocument/'
              f'2006/relationships/styles" Target="styles.xml"/></Relationships>')
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", ct)
        z.writestr("_rels/.rels", rels)
        z.writestr("xl/workbook.xml", wb)
        z.writestr("xl/_rels/workbook.xml.rels", wbrels)
        z.writestr("docProps/app.xml", APP)
        z.writestr("xl/styles.xml", STYLES)
        for i, n in enumerate(names):
            z.writestr(f"xl/worksheets/sheet{i + 1}.xml", sheet_xml(*SHEETS[n]))


if __name__ == "__main__":
    build(sys.argv[1])
