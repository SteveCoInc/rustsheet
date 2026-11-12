"""Builds pivot-style-render.xlsx for the Office.js probe (T-5569, T-5556).

1. Writes the spec (rustsheet-case/v1) of fifteen pivots over one Data sheet.
2. Builds it with the repository's case builder (run from the repository's tools/ directory:
   `uv run --no-dev rustsheet-spec2xlsx -o <out>.xlsx <spec>.json`), which writes
   `PivotStyleLight16` with Excel's switches for a new pivot on every pivot.
3. Rewrites each pivot's `pivotTableStyleInfo` to the style and switches of its sheet; a switch given as None
   is left out of the element (L16S, L16X: what Excel paints for a switch the file does not hold).

Usage: python3 -I make_workbook.py spec <spec.json>      # step 1
       python3 -I make_workbook.py style <built.xlsx> <out.xlsx>   # step 3
"""
import itertools, json, re, sys, zipfile

REGIONS, PRODUCTS, SIZES, CHANNELS, KINDS, REPS = ["East", "West"], ["A", "B"], ["S", "L"], ["Web", "Store"], ["X", "Y"], ["Ann", "Bo"]
HEADER = ["Region", "Product", "Size", "Channel", "Kind", "Rep", "Sales", "Units", "When"]

def records():
    rows = []
    combos = list(itertools.product(REGIONS, PRODUCTS, SIZES, CHANNELS, KINDS))
    for i, (region, product, size, channel, kind) in enumerate(combos):
        if i % 3 == 2:
            continue  # leave some intersections empty
        rows.append([region, product, size, channel, kind, REPS[i % 2], 10 + 7 * i, 1 + i % 3, 45292 + (i % 4) * 31])
    return rows

def data_sheet():
    cells, formats = {}, {}
    for c, name in enumerate(HEADER):
        cells[f"{'ABCDEFGHI'[c]}1"] = name
    for r, row in enumerate(records(), start=2):
        for c, value in enumerate(row):
            cells[f"{'ABCDEFGHI'[c]}{r}"] = value
        formats[f"I{r}"] = "m/d/yyyy"
    return {"name": "Data", "cells": cells, "number_formats": formats}, len(records()) + 1

# sheet, style, showRowHeaders, showColHeaders, showRowStripes, showColStripes, showLastColumn
STYLED = [
    ("L16", "PivotStyleLight16", 1, 1, 0, 0, 1),
    ("L16B", "PivotStyleLight16", 1, 1, 1, 1, 1),
    ("M9", "PivotStyleMedium9", 1, 1, 0, 0, 1),
    ("M9B", "PivotStyleMedium9", 1, 1, 1, 1, 1),
    ("D2", "PivotStyleDark2", 1, 1, 0, 0, 1),
    ("D2B", "PivotStyleDark2", 1, 1, 1, 1, 1),
    ("L16N", "PivotStyleLight16", 0, 0, 0, 0, 0),
    # A switch the file leaves out (None): no stripe switch, as nine pivot parts of the corpus have it, and no
    # switch at all. Microsoft's note on ECMA-376 18.10.1.74 says a missing switch is off and showLastColumn is
    # ignored; L16 (all written), L16N (all off) and these two show whether that holds.
    ("L16S", "PivotStyleLight16", 1, 1, None, None, 1),
    ("L16X", "PivotStyleLight16", None, None, None, None, None),
    # L16 with the last column off alone: whether showLastColumn changes anything.
    ("L16C", "PivotStyleLight16", 1, 1, 0, 0, 0),
]
FORMS = {
    "AC": ({}, {}),
    "AO": ({"compact": False, "compactData": False}, {"compact": False}),
    "AT": ({"compact": False, "outline": False, "compactData": False, "outlineData": False}, {"compact": False, "outline": False}),
}

# Two options the label sheets do not cover (the pivot UI's request P-5): a tabular pivot with "Merge and center
# cells with labels" (`mergeItem`), and a compact pivot whose `indent` is 3.
TABULAR = {"compact": False, "outline": False, "compactData": False, "outlineData": False}
OPTIONS = {
    "AM": (["Region", "Product"], {**TABULAR, "mergeItem": True}, {"compact": False, "outline": False}),
    "AI": (["Region", "Product", "Size"], {"indent": 3}, {}),
}


def spec():
    data, last = data_sheet()
    source = f"Data!A1:I{last}"
    sheets = [data]
    for name, *_ in STYLED:
        definition = {
            "rows": ["Region", "Product", "Size"],
            "columns": ["Channel", "Kind"],
            "pages": [{"field": "Rep"}],
            "data": [{"field": "Sales"}],
            "fields": {"Region": {"attrs": {"insertBlankRow": True}}},
        }
        sheets.append({"name": name, "cells": {}, "pivot_tables": [
            {"name": f"P{name}", "source": source, "location": "A3", "definition": definition}]})
    for name, (table, attrs) in FORMS.items():
        definition = {"rows": ["Region", "Units", "When"], "columns": ["Channel"], "data": [{"field": "Sales"}]}
        if table:
            definition["table"] = table
        if attrs:
            definition["fields"] = {field: {"attrs": dict(attrs)} for field in HEADER}
        sheets.append({"name": name, "cells": {}, "pivot_tables": [
            {"name": f"P{name}", "source": source, "location": "A3", "definition": definition}]})
    for name, (rows, table, attrs) in OPTIONS.items():
        definition = {"rows": rows, "columns": ["Channel"], "data": [{"field": "Sales"}], "table": table}
        if attrs:
            definition["fields"] = {field: {"attrs": dict(attrs)} for field in HEADER}
        sheets.append({"name": name, "cells": {}, "pivot_tables": [
            {"name": f"P{name}", "source": source, "location": "A3", "definition": definition}]})
    return {"format": "rustsheet-case/v1", "description": "Pivot style and label-format render probe (T-5569, T-5556).",
            "tags": ["probe", "pivot"], "sheets": sheets}

def restyle(built, out):
    wanted = {f"P{name}": rest for name, *rest in STYLED}
    source = zipfile.ZipFile(built)
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as target:
        for info in source.infolist():
            content = source.read(info.filename)
            if re.fullmatch(r"xl/pivotTables/pivotTable\d+\.xml", info.filename):
                text = content.decode()
                name = re.search(r'<pivotTableDefinition[^>]* name="([^"]+)"', text).group(1)
                if name in wanted:
                    style, *switches = wanted[name]
                    names = ["showRowHeaders", "showColHeaders", "showRowStripes", "showColStripes", "showLastColumn"]
                    held = "".join(f' {key}="{on}"' for key, on in zip(names, switches) if on is not None)
                    new = f'<pivotTableStyleInfo name="{style}"{held}/>'
                    text, count = re.subn(r"<pivotTableStyleInfo[^>]*/>", new, text)
                    assert count == 1, (name, count)
                    content = text.encode()
            # A fixed timestamp keeps the package byte-deterministic.
            fixed = zipfile.ZipInfo(info.filename, date_time=(2026, 11, 12, 0, 0, 0))
            fixed.compress_type = zipfile.ZIP_DEFLATED
            fixed.external_attr = info.external_attr
            target.writestr(fixed, content)

if sys.argv[1] == "spec":
    with open(sys.argv[2], "w") as f:
        json.dump(spec(), f, indent=1)
        f.write("\n")
elif sys.argv[1] == "style":
    restyle(sys.argv[2], sys.argv[3])
