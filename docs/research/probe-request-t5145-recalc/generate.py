#!/usr/bin/env python3
"""Writes the three workbooks of probe M10 (README.md): files whose cached values are stale, in Manual mode, in
Automatic mode, and in Manual mode with fullCalcOnLoad set.

Standard library only, and the same bytes at every run. Usage: python3 -I generate.py <output directory>
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the probe directories
from probe_package import content_types, output_directory, package_relationships  # noqa: E402
from probe_package import workbook_relationships, write_package  # noqa: E402

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
        ("[Content_Types].xml", content_types(1, legacy=True)),
        ("_rels/.rels", package_relationships(legacy=True)),
        ("xl/workbook.xml", workbook(calc_pr)),
        ("xl/_rels/workbook.xml.rels", workbook_relationships(1)),
        ("xl/worksheets/sheet1.xml", SHEET),
    )
    write_package(path, parts, legacy=True)  # sent before packages were stored and carried app.xml: the bytes stay


def main() -> None:
    out = output_directory()
    # calcId 191029 is the commonest in the repository's Excel-saved fixtures. If the probing build writes a higher
    # one, Excel has a version reason to recalculate the Automatic file on load (README.md, M9 and M10).
    write(out / "m10-manual-stale.xlsx", '<calcPr calcId="191029" calcMode="manual"/>')
    write(out / "m10-auto-stale.xlsx", '<calcPr calcId="191029"/>')
    # ECMA-376: with calcMode manual, "a full recalculation will not be performed on load, even when this attribute
    # is set". Nothing else probes it, and the oracle tool relies on fullCalcOnLoad.
    write(out / "m10-manual-fullcalc.xlsx", '<calcPr calcId="191029" calcMode="manual" fullCalcOnLoad="1"/>')


if __name__ == "__main__":
    main()
