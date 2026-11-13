#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the five workbooks to a scratch directory and reads them back. It says nothing about what Excel does
with them: that is what the request asks. Checked here, against the two tables of README.md:
- the files are the five the README lists, each with the sha256 it gives and every part well-formed XML
  (`listed`, in probe_package.py);
- in each file, cell A i has cell format i, whose font is font i, and the font states exactly what the README
  gives for the cell: for `font-flags.xlsx` the thirteen rows of the second table, for the others the forms of
  its row in the first;
- the cell's text names the form, and B1 counts column A.
"""

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
from probe_package import fail, listed, readme_table  # noqa: E402

MAIN = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
FLAGS = ("b", "i", "strike", "u")


def stated(font) -> str:
    """The font's flag elements as the file has them, such as `<b val="0"/>`."""
    forms = []
    for element in font:
        tag = element.tag.removeprefix(MAIN)
        if tag in FLAGS:
            forms.append(f"<{tag}{''.join(f' {key}="{value}"' for key, value in element.attrib.items())}/>")
    return "".join(forms)


def main() -> None:
    allowed = [
        (row[0], "" if row[1] == "nothing" else row[1].strip("`"))
        for row in readme_table(HERE, "| Cell | The font states |")
    ]
    count = 0
    for (name, described, _), trees in listed(HERE, "| File | Column A |"):
        count += 1
        if name == "font-flags.xlsx":
            wanted = allowed
        else:
            wanted = re.findall(r"(A\d+) `([^`]+)`", described)
        styles, sheet = trees["xl/styles.xml"], trees["xl/worksheets/sheet1.xml"]
        fonts = [stated(font) for font in styles.find(f"{MAIN}fonts")]
        formats = [xf.get("fontId") for xf in styles.find(f"{MAIN}cellXfs")]
        cells = [
            (c.get("r"), c.get("s"), "".join(c.itertext()))
            for c in sheet.iter(f"{MAIN}c")
            if c.get("r")[0] == "A"
        ]
        if [cell[0] for cell in cells] != [cell for cell, _ in wanted]:
            fail(
                f"{name}: column A is {[cell[0] for cell in cells]}, and README.md gives {[c for c, _ in wanted]}"
            )
        for (cell, style, text), (_, form) in zip(cells, wanted):
            font = fonts[int(formats[int(style)])]
            if (style, formats[int(style)], font) != (cell[1:], cell[1:], form):
                fail(
                    f"{name}: {cell} has format {style} and its font states {font!r}, and README.md gives {form!r}"
                )
            if text != (form.strip("</>") or "plain"):
                fail(f"{name}: {cell} holds {text!r}, which does not name {form!r}")
        formulas = [
            (c.get("r"), c.findtext(f"{MAIN}f"))
            for c in sheet.iter(f"{MAIN}c")
            if c.find(f"{MAIN}f") is not None
        ]
        if formulas != [("B1", f"COUNTA(A1:A{len(wanted)})")]:
            fail(f"{name}: the formulas are {formulas}")
    print(f"ok: {count} workbooks: {len(allowed)} forms in one, and {count - 1} values Excel does not write, one a file")


if __name__ == "__main__":
    main()
