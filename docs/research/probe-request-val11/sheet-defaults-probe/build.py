"""Builds sheet-defaults.xlsx: a render probe for what Excel makes of a sheet's <sheetFormatPr>.

Run from the repository's tools/ directory:  uv run python <this file> <output directory>

Written at the XML level, as the other set-1 render probes are (no theme, inline strings, the workbook's
default font Calibri 11), because xlsxwriter always writes defaultRowHeight="15" and never a baseColWidth of
its own. The bytes do not depend on when or where it is built (`package.write_parts`). Column A of every
row says, in OOXML's own words, what that row or sheet holds.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from xml.sax.saxutils import escape

from rustsheet_tools.oracle.package import write_parts

MAIN = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_REL = "http://schemas.openxmlformats.org/package/2006/relationships"
HEAD = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'

STYLES = (
    f'{HEAD}<styleSheet xmlns="{MAIN}"><fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>'
    '<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/>'
    "</fill></fills><borders count=\"1\"><border><left/><right/><top/><bottom/><diagonal/></border></borders>"
    '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
    '<cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>'
    '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>'
)


def text(ref: str, value: str) -> str:
    return f'<c r="{ref}" t="inlineStr"><is><t xml:space="preserve">{escape(value)}</t></is></c>'


def number(ref: str, value: float) -> str:
    return f'<c r="{ref}"><v>{value}</v></c>'


def row(r: int, cells: str, attrs: str = "") -> str:
    return f'<row r="{r}"{" " + attrs if attrs else ""}>{cells}</row>'


def sheet(format_pr: str, rows: list[str]) -> str:
    return (
        f'{HEAD}<worksheet xmlns="{MAIN}"><sheetViews><sheetView workbookViewId="0"></sheetView></sheetViews>'
        f"<sheetFormatPr {format_pr}/><sheetData>{''.join(rows)}</sheetData></worksheet>"
    )


def columns_sheet(format_pr: str) -> tuple[str, str]:
    """A sheet with no <cols>: rows 1 and 2 hold cells, row 3 has no <row>, row 4 one cell in column D."""
    rows = [
        row(1, text("A1", f"sheetFormatPr {format_pr}; no cols") + text("B1", "Sample")),
        row(2, text("A2", "a number in B, C empty") + number("B2", 1234.5)),
        row(4, text("D4", "row 3 has no row element")),
    ]
    return sheet(format_pr, rows), "A1:D4"


def default_height_sheet(format_pr: str) -> tuple[str, str]:
    """Rows 1 and 2 hold cells and no ht; row 3 has no <row>; row 4 is <row/> with no cell and no ht;
    row 5 sets ht=20 customHeight=1 (that a custom height is honoured on this sheet)."""
    rows = [
        row(1, text("A1", f"sheetFormatPr {format_pr}") + text("B1", "Sample")),
        row(2, text("A2", "row with cells, no ht") + number("B2", 1234.5)),
        '<row r="4"/>',
        row(5, text("A5", "ht=20 customHeight=1") + text("B5", "Sample"), 'ht="20" customHeight="1"'),
    ]
    return sheet(format_pr, rows), "A1:B5"


def row_heights_sheet() -> tuple[str, str]:
    """Each stated height twice: as a custom height, and as a height the row merely carries."""
    rows = [row(1, text("A1", 'sheetFormatPr defaultRowHeight="15"; each ht with and without customHeight'))]
    r = 2
    for ht in ("14.4", "15.5", "14", "14.9", "15.1", "16.2", "12.3", "30.1"):
        for custom in (True, False):
            label = f"ht={ht}" + (" customHeight=1" if custom else "")
            attrs = f'ht="{ht}"' + (' customHeight="1"' if custom else "")
            rows.append(row(r, text(f"A{r}", label) + text(f"B{r}", "Sample"), attrs))
            r += 1
    return sheet('defaultRowHeight="15"', rows), f"A1:B{r - 1}"


SHEETS: list[tuple[str, tuple[str, str]]] = [
    # (1) the width of a column with no <col>, from baseColWidth alone; 8 is the schema's default, written out
    ("Base10", columns_sheet('baseColWidth="10" defaultRowHeight="15"')),
    ("Base8", columns_sheet('baseColWidth="8" defaultRowHeight="15"')),
    ("Base5", columns_sheet('baseColWidth="5" defaultRowHeight="15"')),
    ("Base12", columns_sheet('baseColWidth="12" defaultRowHeight="15"')),
    ("Base20", columns_sheet('baseColWidth="20" defaultRowHeight="15"')),
    # ... and beside defaultColWidth: alone, and with a baseColWidth that would give another width
    ("DefCol12", columns_sheet('defaultColWidth="12" defaultRowHeight="15"')),
    ("Base10DefCol12", columns_sheet('baseColWidth="10" defaultColWidth="12" defaultRowHeight="15"')),
    # (2) the height of a row with no ht, from defaultRowHeight with and without customHeight
    ("RowDef14_4", default_height_sheet('defaultRowHeight="14.4"')),
    ("RowDef14_4Custom", default_height_sheet('defaultRowHeight="14.4" customHeight="1"')),
    ("RowDef18", default_height_sheet('defaultRowHeight="18"')),
    ("RowDef18Custom", default_height_sheet('defaultRowHeight="18" customHeight="1"')),
    # (3) how a row's own ht is drawn
    ("RowHt", row_heights_sheet()),
]


def build() -> tuple[bytes, list[dict[str, object]]]:
    count = len(SHEETS)
    overrides = "".join(
        f'<Override PartName="/xl/worksheets/sheet{n}.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>'
        for n in range(1, count + 1)
    )
    parts = {
        "[Content_Types].xml": (
            f'{HEAD}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Default Extension="xml" ContentType="application/xml"/>'
            '<Override PartName="/xl/workbook.xml" '
            'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>'
            '<Override PartName="/xl/styles.xml" '
            'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>'
            f"{overrides}</Types>"
        ),
        "_rels/.rels": (
            f'{HEAD}<Relationships xmlns="{PKG_REL}"><Relationship Id="rId1" '
            f'Type="{REL}/officeDocument" Target="xl/workbook.xml"/></Relationships>'
        ),
        "xl/workbook.xml": (
            f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets>'
            + "".join(
                f'<sheet name="{escape(name)}" sheetId="{n}" r:id="rId{n}"/>'
                for n, (name, _) in enumerate(SHEETS, 1)
            )
            + '</sheets><calcPr fullCalcOnLoad="1"/></workbook>'
        ),
        "xl/_rels/workbook.xml.rels": (
            f'{HEAD}<Relationships xmlns="{PKG_REL}">'
            + "".join(
                f'<Relationship Id="rId{n}" Type="{REL}/worksheet" Target="worksheets/sheet{n}.xml"/>'
                for n in range(1, count + 1)
            )
            + f'<Relationship Id="rId{count + 1}" Type="{REL}/styles" Target="styles.xml"/></Relationships>'
        ),
        "xl/styles.xml": STYLES,
    }
    for n, (_, (xml, _range)) in enumerate(SHEETS, 1):
        parts[f"xl/worksheets/sheet{n}.xml"] = xml
    manifest = [{"name": name, "range": cells, "charts": 0} for name, (_, cells) in SHEETS]
    return write_parts({name: data.encode() for name, data in parts.items()}), manifest


if __name__ == "__main__":
    out = Path(sys.argv[1])
    xlsx, sheets = build()
    (out / "sheet-defaults.xlsx").write_bytes(xlsx)
    entry = {
        "id": "sheet-defaults",
        "file": "sheet-defaults.xlsx",
        "sha256": hashlib.sha256(xlsx).hexdigest(),
        "sheets": sheets,
    }
    (out / "probes-entry.json").write_text(json.dumps(entry, indent=1) + "\n")
    print(entry["sha256"], len(xlsx), "bytes,", len(sheets), "sheets")
