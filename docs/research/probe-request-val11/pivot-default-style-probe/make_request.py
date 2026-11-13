"""Builds the three workbooks of the pivot-default-style probe.

1. `spec <spec.json>`: writes the spec (rustsheet-case/v1): a Data sheet and an empty sheet New.
2. Build it with the repository's case builder, from the repository's tools/ directory:
   `uv run --no-dev rustsheet-spec2xlsx -o <built>.xlsx <spec>.json`.
3. `styles <built.xlsx> <out-dir>`: writes three copies whose styles part differs only in `tableStyles`
   (and, for the third, one `dxf`): the default pivot style is PivotStyleMedium9, absent, or a pivot style the
   workbook defines.
"""
import json, re, sys, zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))  # probe_package.py
from probe_package import copy_entry  # noqa: E402

ROWS = [("East", "A", 10), ("East", "B", 20), ("West", "A", 30), ("West", "B", 40), ("North", "A", 50), ("North", "B", 60)]

def spec():
    cells = {"A1": "Region", "B1": "Product", "C1": "Sales"}
    for r, (region, product, sales) in enumerate(ROWS, start=2):
        cells[f"A{r}"], cells[f"B{r}"], cells[f"C{r}"] = region, product, sales
    return {"format": "rustsheet-case/v1", "description": "Which style Excel for the web gives a new pivot (T-5569).",
            "tags": ["probe", "pivot"], "sheets": [{"name": "Data", "cells": cells}, {"name": "New", "cells": {"H1": "probe"}}]}

VARIANTS = {
    "m9": ('<tableStyles count="0" defaultTableStyle="TableStyleMedium2" defaultPivotStyle="PivotStyleMedium9"/>', None),
    "none": ('<tableStyles count="0" defaultTableStyle="TableStyleMedium2"/>', None),
    "own": ('<tableStyles count="1" defaultTableStyle="TableStyleMedium2" defaultPivotStyle="ProbePivot">'
            '<tableStyle name="ProbePivot" table="0" count="1"><tableStyleElement type="headerRow" dxfId="0"/></tableStyle>'
            '</tableStyles>',
            '<dxfs count="1"><dxf><fill><patternFill patternType="solid"><fgColor rgb="FFFFC000"/><bgColor rgb="FFFFC000"/>'
            '</patternFill></fill></dxf></dxfs>'),
}

def styles(built, out_dir):
    source = zipfile.ZipFile(built)
    for key, (table_styles, dxfs) in VARIANTS.items():
        with zipfile.ZipFile(f"{out_dir}/pivot-default-style-{key}.xlsx", "w", zipfile.ZIP_DEFLATED) as target:
            for info in source.infolist():
                content = source.read(info.filename)
                if info.filename == "xl/styles.xml":
                    text = content.decode()
                    text, n = re.subn(r"<tableStyles[^>]*/>|<tableStyles[^>]*>.*?</tableStyles>", table_styles, text, flags=re.S)
                    assert n == 1, ("tableStyles", n)
                    if dxfs:
                        text, n = re.subn(r"<dxfs[^>]*/>|<dxfs[^>]*>.*?</dxfs>", dxfs, text, flags=re.S)
                        assert n == 1, ("dxfs", n)
                    content = text.encode()
                copy_entry(target, info, content)

if sys.argv[1] == "spec":
    with open(sys.argv[2], "w") as f:
        json.dump(spec(), f, indent=1)
        f.write("\n")
elif sys.argv[1] == "styles":
    styles(sys.argv[2], sys.argv[3])
