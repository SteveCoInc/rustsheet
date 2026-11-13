#!/usr/bin/env python3
"""Writes the workbooks of the probe request (README.md).

For the READ case ("By hand"), two files that say precision as displayed is on (`calcPr fullPrecision="0"`)
while their cells hold more digits than their formats show, as a writer other than Excel can leave them: A1 =
1.23456, a constant in format 0.00. B1 = A1*1000, General, cached 1234.56. C1 = the formula =1.23456 in format
0.00, cached 1.23456. D1 = C1*1000, General, cached 1234.56.

For excel-graph ("Through excel-graph"), the table of README.md as a file, with the option on and, as a
control, off: row for row, the number as a constant in column A and its formula in column B, both in the row's
format, C and D reading them in General, and one Ctrl+Shift+Enter array over F2:F3 whose two cells have
different formats. The formula cells hold no cached value.

Standard library only, and the same bytes at every run. Usage: python3 -I generate.py <output directory>
"""
import re
import sys
from pathlib import Path
from xml.sax.saxutils import escape, quoteattr

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py, beside the directories
from probe_package import HEAD, MAIN, OFFICE, PACKAGE_RELATIONSHIPS  # noqa: E402
from probe_package import content_types, output_directory, styles, workbook_relationships  # noqa: E402
from probe_package import write_package  # noqa: E402

# Cell format 0 is General; cell format 1 is the built-in number format 2, "0.00".
STYLES = styles(cell_formats=(2,))
SHEET = (
    f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData><row r="1">'
    '<c r="A1" s="1"><v>1.23456</v></c>'
    '<c r="B1"><f>A1*1000</f><v>1234.56</v></c>'
    '<c r="C1" s="1"><f>1.23456</f><v>1.23456</v></c>'
    '<c r="D1"><f>C1*1000</f><v>1234.56</v></c>'
    "</row></sheetData></worksheet>"
)


def table_rows() -> list[tuple[int, str, str, str]]:
    """(sheet row, format, number, formula) for every row README.md lists under "The rows of the table"."""
    readme = (Path(__file__).resolve().parent / "README.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| (\d+) \| `(.*)` \| (\S+) \| `=(.*)` \|$", readme, flags=re.MULTILINE)
    if not rows or [int(r[0]) for r in rows] != list(range(2, len(rows) + 2)):
        sys.exit("README.md does not list the rows of the table from sheet row 2 on")
    return [(int(row), code, number, formula) for row, code, number, formula in rows]


def table_parts(calc_pr: str) -> tuple[tuple[str, str], ...]:
    """The package of the table: cell format 0 is General, and every other format code has one of its own."""
    rows = table_rows()
    codes = list(dict.fromkeys(code for _, code, _, _ in rows if code != "General"))
    xf = {"General": 0, **{code: i + 1 for i, code in enumerate(codes)}}
    own = tuple((164 + i, quoteattr(code)) for i, code in enumerate(codes))
    table_styles = styles(cell_formats=tuple(number for number, _ in own), number_formats=own)
    sheet = [f'{HEAD}<worksheet xmlns="{MAIN}"><sheetData>']
    for row, code, number, formula in rows:
        sheet.append(
            f'<row r="{row}">'
            f'<c r="A{row}" s="{xf[code]}"><v>{float(number)!r}</v></c>'
            f'<c r="B{row}" s="{xf[code]}"><f>{escape(formula)}</f></c>'
            f'<c r="C{row}"><f>A{row}</f></c><c r="D{row}"><f>B{row}</f></c>'
        )
        # One Ctrl+Shift+Enter array over F2:F3: its first cell in the format of row 2, its second in General.
        # It divides by 7, so that each member has more digits than `0.00` shows whether or not Excel has
        # rounded the constants it reads.
        if row == 2:
            sheet.append(f'<c r="F2" s="{xf[code]}"><f t="array" ref="F2:F3">A2:A3/7</f></c>')
        if row == 3:
            sheet.append('<c r="F3"/>')
        sheet.append("</row>")
    sheet.append("</sheetData></worksheet>")
    return package_parts(calc_pr, table_styles, "".join(sheet))


def workbook(calc_pr: str) -> str:
    return (
        f'{HEAD}<workbook xmlns="{MAIN}" xmlns:r="{OFFICE}">'
        '<sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>' + calc_pr + "</workbook>"
    )


def package_parts(calc_pr: str, styles_part: str, sheet: str) -> tuple[tuple[str, str], ...]:
    """A workbook of one sheet, S, with its styles part."""
    return (
        ("[Content_Types].xml", content_types(1, styles=True)),
        ("_rels/.rels", PACKAGE_RELATIONSHIPS),
        ("xl/workbook.xml", workbook(calc_pr)),
        ("xl/_rels/workbook.xml.rels", workbook_relationships(1, styles=True)),
        ("xl/styles.xml", styles_part),
        ("xl/worksheets/sheet1.xml", sheet),
    )


def write(path: Path, calc_pr: str) -> None:
    write_package(path, package_parts(calc_pr, STYLES, SHEET))


def main() -> None:
    out = output_directory()
    # calcId 191029 is the commonest in the repository's Excel-saved fixtures. A build that writes a higher one
    # has a version reason to recalculate on load, so the second file asks for the recalculation outright.
    write(out / "pad-unrounded.xlsx", '<calcPr calcId="191029" fullPrecision="0"/>')
    write(out / "pad-unrounded-fullcalc.xlsx", '<calcPr calcId="191029" fullPrecision="0" fullCalcOnLoad="1"/>')
    # The table, for excel-graph: the oracle tool calculates fully whatever the file says; the attribute is for
    # a reader that opens the file by hand.
    on = '<calcPr calcId="191029" fullPrecision="0" fullCalcOnLoad="1"/>'
    write_package(out / "pad-table.xlsx", table_parts(on))
    write_package(out / "pad-table-control.xlsx", table_parts('<calcPr calcId="191029" fullCalcOnLoad="1"/>'))


if __name__ == "__main__":
    main()
