#!/usr/bin/env python3
"""Counts, over every xlsx and xlsm under the given roots, the chart parts and the series formulas in them that
are not a cell or a range of cells, and prints each such formula with its file's name.

Chart parts are found as the package links them: the target of a relationship of the chart type, from any part,
whatever the part is called. A formula is an `f` element of the chart namespace, whatever prefix the part gives
that namespace. The package is read by the oracle tooling's own reader
(tools/src/rustsheet_tools/oracle/package.py). A file that is no readable package, and a chart part that is
linked and is missing or is not XML, are counted and listed, so that a scan that missed something says so.

It reads workbooks it did not write: run it with -I. Usage: python3 -I scan.py [--no-names] <root> [<root> ...]
"""

import posixpath
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "tools" / "src"))
from rustsheet_tools.oracle.package import parse_part, read_parts  # noqa: E402

CELL = r"(?:(?:'[^']*'|[A-Za-z0-9_.]+)!)?\$?[A-Z]{1,3}\$?[0-9]+(?::\$?[A-Z]{1,3}\$?[0-9]+)?"
# One area or a union of areas, as a series of several ranges is written.
CELLS = re.compile(rf"\(?{CELL}(?:,{CELL})*\)?")
# The chart namespace and the chart relationship, as a transitional and as a strict package name them.
FORMULAS = {
    "{http://schemas.openxmlformats.org/drawingml/2006/chart}f",
    "{http://purl.oclc.org/ooxml/drawingml/chart}f",
}
CHART_LINKS = {
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/chart",
}
RELATIONSHIP = "{http://schemas.openxmlformats.org/package/2006/relationships}Relationship"


def chart_parts(parts: dict[str, bytes], unread: list[str]) -> list[str]:
    """The parts a chart relationship names, in the order met; a relationships part that is not XML is unread."""
    found: dict[str, None] = {}
    for name, data in parts.items():
        if not name.endswith(".rels"):
            continue
        try:
            links = list(parse_part(name, data).iter(RELATIONSHIP))
        except Exception:  # noqa: BLE001 - whatever the reader raises, the part was not read
            unread.append(name)
            continue
        # `a/_rels/b.xml.rels` holds the relationships of `a/b.xml`: its targets are relative to `a`.
        directory = posixpath.dirname(posixpath.dirname(name))
        for link in links:
            if link.get("Type") in CHART_LINKS and link.get("TargetMode") != "External":
                target = link.get("Target", "")
                relative = posixpath.normpath(posixpath.join(directory, target))
                found[target[1:] if target.startswith("/") else relative] = None
    return list(found)


def main() -> None:
    show = "--no-names" not in sys.argv
    roots = [a for a in sys.argv[1:] if not a.startswith("--")]
    workbooks = charts = 0
    other: list[tuple[str, str]] = []
    unread: list[tuple[str, str]] = []  # (workbook, a part that is linked and was not read)
    closed: list[str] = []  # files with a workbook's extension that are no readable package
    for root in roots:
        for path in sorted(p for p in Path(root).rglob("*") if p.suffix.lower() in (".xlsx", ".xlsm")):
            if not path.is_file():
                continue
            try:
                parts = read_parts(path.read_bytes())
            except Exception:  # noqa: BLE001 - a zip's codecs raise their own errors; none is a package read
                closed.append(path.name)
                continue
            workbooks += 1
            missed: list[str] = []
            for part in chart_parts(parts, missed):
                try:
                    tree = parse_part(part, parts[part])
                except Exception:  # noqa: BLE001 - missing (KeyError) or not XML: linked and not read
                    missed.append(part)
                    continue
                charts += 1
                for element in tree.iter():
                    formula = element.text or ""
                    if element.tag in FORMULAS and not CELLS.fullmatch(formula):
                        other.append((path.name, formula))
            unread += [(path.name, part) for part in missed]
    print(f"workbooks read: {workbooks}; not readable: {len(closed)}; chart parts: {charts}")
    print(f"series formulas that are not cells: {len(other)}")
    for name, formula in other:
        print(f"  {name}: {formula}" if show else f"  {formula}")
    if unread or closed:
        print(f"INCOMPLETE: {len(unread)} part(s) linked and not read; {len(closed)} file(s) not readable")
        if show:
            for name, what in [*unread, *((name, "not a readable package") for name in closed)][:40]:
                print("     ", name, what)


if __name__ == "__main__":
    main()
