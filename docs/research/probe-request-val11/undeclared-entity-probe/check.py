#!/usr/bin/env python3
"""Checks generate.py without Excel: python3 -I check.py

It writes the eight workbooks to a scratch directory and reads them back. It says nothing about what Excel does
with them: that is what the request asks. Checked here:
- the two parts taken from an Excel-written workbook of the repository are the ones the sha256 values of
  README.md were made from (a re-save of that workbook would change every one of them);
- every part of the control is well-formed XML, and it holds the marker values README.md names: a row height,
  a font and a colour scheme that are not Excel's defaults;
- each case differs from the control in exactly one part, the one README.md names, and that part is the
  control's with one `&nbsp;` added in the text README.md gives;
- that part is no longer well-formed XML, and every other part of the case still is;
- every sha256 README.md gives is its file's.
"""
import hashlib
import re
import sys
import zipfile
from pathlib import Path
from xml.etree import ElementTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))  # probe_package.py, beside the directories
sys.path.insert(0, str(HERE))  # generate.py: the source workbook's place has one definition
from generate import SOURCE  # noqa: E402
from probe_package import fail, generated  # noqa: E402

# The parts generate.py takes from the Excel-written workbook, by the sha256 they had when README.md was written.
SOURCE_PARTS = {
    "xl/theme/theme1.xml": "e6029ab4958414b8bb862b17ffed3a708d1513e61a07d88e966071cca31d1bd4",
    "docProps/app.xml": "09568d58402749db517a54e99abde6df4e8090f403cb8d85fe4d02d57bc4c7ed",
}
# What the control holds where a case asks what Excel kept: the part, and the text README.md names.
MARKERS = """
xl/worksheets/sheet1.xml | <row r="1" ht="30" customHeight="1">
xl/styles.xml | <sz val="13"/><name val="Courier New"/>
xl/theme/theme1.xml | <a:clrScheme name="Probe">
xl/theme/theme1.xml | <a:accent1><a:srgbClr val="123456"/></a:accent1>
"""

# File | the part that holds the entity | the text around it, as README.md shows it | the control's text there.
CASES = """
entity-sheet-text.xlsx | xl/worksheets/sheet1.xml | <t>a&nbsp;b</t> | <t>a b</t>
entity-sheet-attribute.xlsx | xl/worksheets/sheet1.xml | ht="30&nbsp;" | ht="30"
entity-shared-strings.xlsx | xl/sharedStrings.xml | <t>c&nbsp;d</t> | <t>c d</t>
entity-styles.xlsx | xl/styles.xml | <name val="Courier New&nbsp;"/> | <name val="Courier New"/>
entity-theme.xlsx | xl/theme/theme1.xml | <a:clrScheme name="Probe&nbsp;"> | <a:clrScheme name="Probe">
entity-app.xlsx | docProps/app.xml | <Application>Microsoft&nbsp;Excel | <Application>Microsoft Excel
entity-core.xlsx | docProps/core.xml | <dc:title>Probe&nbsp;title | <dc:title>Probe title
"""


def well_formed(data: bytes) -> bool:
    try:
        ElementTree.fromstring(data)
    except ElementTree.ParseError:
        return False
    return True


def main() -> None:
    readme = re.sub(r"\s+", " ", (HERE / "README.md").read_text(encoding="utf-8"))
    with zipfile.ZipFile(SOURCE) as source:
        for part, sha in SOURCE_PARTS.items():
            if hashlib.sha256(source.read(part)).hexdigest() != sha:
                fail(f"{SOURCE.name}'s {part} has changed: every sha256 of README.md was made from the earlier one")
    packages = generated(HERE)
    cases = [line.split(" | ") for line in CASES.strip().splitlines()]
    names = sorted(["entity-control.xlsx"] + [case[0] for case in cases])
    if sorted(packages) != names:
        fail(f"generate.py wrote {sorted(packages)}, not {names}")
    control = packages["entity-control.xlsx"][1]
    for part, data in control.items():
        if not well_formed(data):
            fail(f"the control's {part} is not well-formed XML")
    for part, marker in (line.split(" | ") for line in MARKERS.strip().splitlines()):
        if control[part].decode("utf-8").count(marker) != 1:
            fail(f"the control's {part} does not hold {marker} once")
    for name, part, text, plain in cases:
        parts = packages[name][1]
        differing = [p for p in control if parts.get(p) != control[p]]
        if list(parts) != list(control) or differing != [part]:
            fail(f"{name} differs from the control in {differing}, not in {part} alone")
        changed = parts[part].decode("utf-8")
        if changed.count("&nbsp;") != 1 or text not in changed:
            fail(f"{name}: {part} does not hold {text} with one entity")
        if changed.replace(text, plain) != control[part].decode("utf-8"):
            fail(f"{name}: {part} is not the control's part with the entity added")
        if well_formed(parts[part]):
            fail(f"{name}: {part} is still well-formed XML")
        if text not in readme:
            fail(f"README.md does not show {text} for {name}")
    # Last, so that a change of a workbook is first told by what changed in it.
    for name, (sha, _) in packages.items():
        if f"`{name}` (sha256 `{sha}`)" not in readme:
            fail(f"README.md does not name {name} with its sha256 {sha}")
    print(f"ok: {len(packages)} workbooks, a control and {len(cases)} cases with one undeclared entity each")


if __name__ == "__main__":
    main()
