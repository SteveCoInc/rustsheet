#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the six workbooks to a scratch directory and reads them back. It says nothing about what Excel does
with them: that is what the request asks. Checked here, against the two tables of README.md:
- the files are the six the README lists, each with the sha256 it gives, every part well-formed XML and every
  relationship's target a part of the package (`listed`, in probe_package.py);
- the sheet holds the README's cells;
- the three names are the workbook's or the sheet's, as the README says of the file, and name the cells it
  gives;
- the chart's series gives its name, categories and values by the three formulas the README lists for the
  file, in that order, and each cache holds the stale values, none of which is in a cell.
"""

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import fail, listed, readme_table  # noqa: E402

MAIN = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
CHART = "{http://schemas.openxmlformats.org/drawingml/2006/chart}"
NAMES = {"Label": "Sheet1!$B$1", "Months": "Sheet1!$A$2:$A$5", "Sales": "Sheet1!$B$2:$B$5"}
STALE = (["stale"], ["w", "x", "y", "z"], ["901", "902", "903", "904"])
SCOPES = {"the workbook's": {None}, "Sheet1's": {"0"}}


def main() -> None:
    expected_cells = [row[1:] for row in readme_table(HERE, "| | A | B | C |")]
    in_cells = {value for row in expected_cells for value in row}
    count = 0
    for (name, scope, formulas, _), trees in listed(HERE, "| File | The names are |"):
        count += 1
        rows = trees["xl/worksheets/sheet1.xml"].iter(f"{MAIN}row")
        if [["".join(cell.itertext()) for cell in row.iter(f"{MAIN}c")] for row in rows] != expected_cells:
            fail(f"{name}: the sheet's cells are not README.md's")
        defined = trees["xl/workbook.xml"].findall(f"{MAIN}definedNames/{MAIN}definedName")
        if {item.get("name"): item.text for item in defined} != NAMES:
            fail(f"{name}: the names are {[(item.get('name'), item.text) for item in defined]}")
        local = {item.get("localSheetId") for item in defined}
        if local != SCOPES.get(scope):
            fail(f"{name}: README.md says the names are {scope}, and the file has localSheetId {local}")
        series = trees["xl/charts/chart1.xml"].findall(f".//{CHART}ser")
        if len(series) != 1:
            fail(f"{name}: the chart has {len(series)} series")
        found = [series[0].find(f"{CHART}{tag}//{CHART}f").text for tag in ("tx", "cat", "val")]
        if found != re.findall(r"`([^`]+)`", formulas):
            fail(f"{name}: the series' formulas are {found}, and README.md gives {formulas}")
        caches = [
            [point.find(f"{CHART}v").text for point in series[0].find(f"{CHART}{tag}").iter(f"{CHART}pt")]
            for tag in ("tx", "cat", "val")
        ]
        if tuple(caches) != STALE or in_cells & {value for cache in caches for value in cache}:
            fail(f"{name}: the caches are {caches}")
    print(f"ok: {count} workbooks as README.md lists them")


if __name__ == "__main__":
    main()
