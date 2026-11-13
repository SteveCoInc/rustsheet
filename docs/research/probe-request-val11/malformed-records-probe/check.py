#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the six workbooks to a scratch directory and reads them back. It says nothing about what Excel does
with them: that is what the request asks. Checked here:
- row-repeated.xlsx holds two records for row 2, in the order and with the attributes and cells README.md
  gives, and its control the first record alone;
- col-overlap.xlsx holds the two `<col>` records, the second inside the first, and its control the first alone;
- hyperlink-reversed.xlsx holds one hyperlink whose `ref` is `B2:A1`, and its control `A1:B2`;
- a case and its control are the same package but for that one difference;
- each sheet holds the formulas README.md names;
- every sha256 README.md gives is its file's.
"""
import re
import sys
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import MAIN, fail, generated  # noqa: E402

NS = {"m": MAIN}


def expect(got: object, want: object, what: str) -> None:
    if got != want:
        fail(f"{what}: {got!r}, not {want!r}")


def main() -> None:
    readme = re.sub(r"\s+", " ", (HERE / "README.md").read_text(encoding="utf-8"))
    packages = generated(HERE)
    cases = ["row-repeated", "col-overlap", "hyperlink-reversed"]
    names = sorted(f"{case}{suffix}.xlsx" for case in cases for suffix in ("", "-control"))
    expect(sorted(packages), names, "the files")

    sheets = {}
    for name, (_, parts) in packages.items():
        for part, data in parts.items():
            ElementTree.fromstring(data)  # every part is well-formed XML
        sheets[name] = ElementTree.fromstring(parts["xl/worksheets/sheet1.xml"])
    for case in cases:
        case_parts, control_parts = packages[f"{case}.xlsx"][1], packages[f"{case}-control.xlsx"][1]
        differing = [part for part in case_parts if case_parts[part] != control_parts.get(part)]
        expect(differing, ["xl/worksheets/sheet1.xml"], f"{case}: the parts that differ from the control")

    def rows(name: str) -> list[tuple[dict[str, str], list[tuple[str | None, str | None]]]]:
        found = []
        for row in sheets[name].iterfind("m:sheetData/m:row", NS):
            constants = [c for c in row.iterfind("m:c", NS) if c.find("m:f", NS) is None]
            found.append((dict(row.attrib), [(c.get("r"), c.findtext("m:v", None, NS)) for c in constants]))
        return found

    kept = {"r": "2", "ht": "30", "customHeight": "1", "s": "1", "customFormat": "1"}
    first = (kept, [("A2", "10")])
    second = (
        {"r": "2", "ht": "60", "customHeight": "1", "hidden": "1", "s": "2", "customFormat": "1"},
        [("A2", "20"), ("B2", "5")],
    )
    row_1, row_3 = ({"r": "1"}, [("A1", "1")]), ({"r": "3"}, [("A3", "100")])
    expect(rows("row-repeated.xlsx"), [row_1, first, second, row_3], "row-repeated.xlsx: its rows")
    # The control: the first record's attributes, and the cells as the second record leaves them.
    as_imported = (kept, [("A2", "20"), ("B2", "5")])
    control = rows("row-repeated-control.xlsx")
    expect(control, [row_1, as_imported, row_3], "row-repeated-control.xlsx: its rows")

    def cols(name: str) -> list[dict[str, str]]:
        return [dict(col.attrib) for col in sheets[name].iterfind("m:cols/m:col", NS)]

    wide = {"min": "1", "max": "5", "width": "20", "customWidth": "1"}
    inner = {"min": "2", "max": "3", "width": "40", "customWidth": "1", "hidden": "1"}
    expect(cols("col-overlap.xlsx"), [wide, inner], "col-overlap.xlsx: its col records")
    expect(cols("col-overlap-control.xlsx"), [wide], "col-overlap-control.xlsx: its col records")

    def links(name: str) -> list[dict[str, str]]:
        return [dict(link.attrib) for link in sheets[name].iterfind("m:hyperlinks/m:hyperlink", NS)]

    link = {"location": "S!D5", "display": "x"}
    expect(links("hyperlink-reversed.xlsx"), [{"ref": "B2:A1", **link}], "hyperlink-reversed.xlsx: its link")
    expect(links("hyperlink-reversed-control.xlsx"), [{"ref": "A1:B2", **link}], "the control's link")

    formulas = {
        "row-repeated": {
            "D1": "A2+B2",
            "E1": "SUBTOTAL(109,A1:A3)",
            "F1": "_xlfn.AGGREGATE(9,5,A1:A3)",
            "G1": "SUM(A1:B3)",
        },
        "col-overlap": {"G1": "SUM(A1:C1)"},
        "hyperlink-reversed": {"G1": "SUM(A1:B2)"},
    }
    for case, want in formulas.items():
        for name in (f"{case}.xlsx", f"{case}-control.xlsx"):
            cells = [c for c in sheets[name].iterfind(".//m:c", NS) if c.find("m:f", NS) is not None]
            got = {c.get("r"): c.findtext("m:f", None, NS) for c in cells}
            expect(got, want, f"{name}: its formulas")
            for address, formula in want.items():
                # A file stores a function newer than Excel 2007 with a prefix the README does not show.
                shown = formula.replace("_xlfn.", "")
                if f"{address} `={shown}`" not in readme:
                    fail(f"README.md does not name {address} `={shown}` for {case}")

    # Last, so that a change of a workbook is first told by what changed in it.
    for name, (sha, _) in packages.items():
        if f"`{name}` (sha256 `{sha}`)" not in readme:
            fail(f"README.md does not name {name} with its sha256 {sha}")
    print(f"ok: {len(packages)} workbooks, three cases and their controls")


if __name__ == "__main__":
    main()
