#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the ten workbooks to a scratch directory and reads them back. It says nothing about what Excel does
with them: that is what the request asks. Checked here, against the table of README.md:
- the files are the ten the README lists, each with the sha256 it gives, every part well-formed XML and every
  relationship's target a part of the package (`listed`, in probe_package.py);
- the sheet's table part is `Probe` over A1:B3 with the columns `k` and `n`, and D1 holds `SUM(Probe[n])`;
- the table's `<tableStyleInfo>` has exactly the attributes the README gives for the file, with those values,
  or is absent where the README says there is none;
- the styles part has a `<tableStyles>` element, defining `ProbeStyle`, in the one file whose row says so, and
  in no other.
"""

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import fail, listed  # noqa: E402

MAIN = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
ATTRIBUTE = re.compile(r'([A-Za-z]+)="([^"]*)"')


def main() -> None:
    count, above = 0, ""
    for (name, described, _), trees in listed(HERE, "| File | `<tableStyleInfo>` |"):
        count += 1
        table = trees["xl/tables/table1.xml"]
        columns = [column.get("name") for column in table.iter(f"{MAIN}tableColumn")]
        if (table.get("name"), table.get("ref"), columns) != ("Probe", "A1:B3", ["k", "n"]):
            fail(f"{name}: the table is {table.get('name')} over {table.get('ref')} with {columns}")
        formulas = [f.text for f in trees["xl/worksheets/sheet1.xml"].iter(f"{MAIN}f")]
        if formulas != ["SUM(Probe[n])"]:
            fail(f"{name}: the sheet's formulas are {formulas}")
        # A row gives the element's attributes in its first code span; "the same" repeats the row above.
        spans = re.findall(r"`([^`]+)`", described)
        attributes = (
            above if described.startswith("the same") else spans[0] if spans and "=" in spans[0] else None
        )
        info = [list(element.attrib.items()) for element in table.findall(f"{MAIN}tableStyleInfo")]
        wanted = [] if described.startswith("none") else [ATTRIBUTE.findall(attributes or "")]
        if info != wanted or (attributes is None and not described.startswith("none")):
            fail(f"{name}: <tableStyleInfo> is {info}, and README.md gives {described}")
        defines = ["ProbeStyle"] if "defines `ProbeStyle`" in described else []
        styles = trees["xl/styles.xml"].findall(f"{MAIN}tableStyles/{MAIN}tableStyle")
        if [style.get("name") for style in styles] != defines:
            fail(f"{name}: the styles part defines {[style.get('name') for style in styles]}")
        above = attributes or above
    print(f"ok: {count} workbooks as README.md lists them")


if __name__ == "__main__":
    main()
