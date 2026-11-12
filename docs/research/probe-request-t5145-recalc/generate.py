#!/usr/bin/env python3
"""Writes the three workbooks of probe M10 (README.md): files whose cached values are stale, in Manual mode, in
Automatic mode, and in Manual mode with fullCalcOnLoad set.

Standard library only, and the same bytes at every run. Usage: python3 -I generate.py <output directory>
"""
import hashlib
import sys
import zipfile
from pathlib import Path

CONTENT_TYPES = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
    '<Default Extension="xml" ContentType="application/xml"/>'
    '<Override PartName="/xl/workbook.xml" '
    'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>'
    '<Override PartName="/xl/worksheets/sheet1.xml" '
    'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>'
    "</Types>"
)
RELS = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    '<Relationship Id="rId1" '
    'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" '
    'Target="xl/workbook.xml"/></Relationships>'
)
WORKBOOK_RELS = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    '<Relationship Id="rId1" '
    'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" '
    'Target="worksheets/sheet1.xml"/></Relationships>'
)
# A1 = 1 (a number); B1 = A1+1 with the cached value 999 (stale on purpose); C1 = B1*10 cached 9990.
SHEET = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>'
    '<row r="1"><c r="A1"><v>1</v></c><c r="B1"><f>A1+1</f><v>999</v></c>'
    '<c r="C1"><f>B1*10</f><v>9990</v></c></row>'
    "</sheetData></worksheet>"
)


def workbook(calc_pr: str) -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" '
        'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
        '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>' + calc_pr + "</workbook>"
    )


def write(path: Path, calc_pr: str) -> None:
    parts = (
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", RELS),
        ("xl/workbook.xml", workbook(calc_pr)),
        ("xl/_rels/workbook.xml.rels", WORKBOOK_RELS),
        ("xl/worksheets/sheet1.xml", SHEET),
    )
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, text in parts:
            # A fixed timestamp, so that two runs write the same bytes and the request can name a sha256.
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3  # zipfile writes 0 on Windows: keep the bytes the same on every platform
            z.writestr(info, text)
    print(hashlib.sha256(path.read_bytes()).hexdigest(), path.name)


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit("usage: python3 -I generate.py <output directory>")
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    # calcId 191029 is the commonest in the repository's Excel-saved fixtures. If the probing build writes a higher
    # one, Excel has a version reason to recalculate the Automatic file on load (README.md, M9 and M10).
    write(out / "m10-manual-stale.xlsx", '<calcPr calcId="191029" calcMode="manual"/>')
    write(out / "m10-auto-stale.xlsx", '<calcPr calcId="191029"/>')
    # ECMA-376: with calcMode manual, "a full recalculation will not be performed on load, even when this attribute
    # is set". Nothing else probes it, and the oracle tool relies on fullCalcOnLoad.
    write(out / "m10-manual-fullcalc.xlsx", '<calcPr calcId="191029" calcMode="manual" fullCalcOnLoad="1"/>')


if __name__ == "__main__":
    main()
