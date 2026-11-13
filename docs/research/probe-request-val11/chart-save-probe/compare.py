#!/usr/bin/env python3
"""Says what a save did to each chart of a workbook, as a Markdown table for RESULTS.md.

    python3 -I compare.py <before.xlsx> <after.xlsx>
    python3 -I compare.py <directory of inputs> <directory of saved files>

With two directories, `name.xlsx` of the first is compared with `name.web.xlsx` of the second.

A chart is found the way Excel finds it, from its sheet: sheet, drawing, frame, chart part. So a chart is the
same chart before and after even if the save gave its part another name. For each chart the table says:

- the chart part: `identical`; `ids only` if the two differ in nothing but series ids (`c16:uniqueId`), with how
  many of them changed; `differs`, with the first differences listed under the table; `gone`;
- its style part and its colours part: `identical`, `differs`, `gone`, or `none` if the chart had none;
- its other related parts, each by the last word of its relationship type;
- its frame, the anchor element of the drawing that holds it: `identical` or `differs`.

Standard library only. The workbooks are read, never extracted or run. Relationship parts are read with regular
expressions, not an XML parser, so that an entity in an input costs nothing.
"""

import difflib
import html
import posixpath
import re
import sys
import zipfile
from pathlib import Path

ATTRIBUTE = re.compile(r'([\w:.-]+)\s*=\s*"([^"]*)"')
RELATIONSHIP = re.compile(r"<(?:\w+:)?Relationship\b([^>]*)>")
SHEET = re.compile(r"<(?:\w+:)?sheet\b([^>]*)>")
ANCHOR = re.compile(r"<((?:\w+:)?(?:twoCellAnchor|oneCellAnchor|absoluteAnchor))\b.*?</\1>", re.DOTALL)
REFERENCE = re.compile(r'\b\w+:id\s*=\s*"([^"]*)"')
SERIES_ID = re.compile(rb'(<c16:uniqueId\b[^>]*\bval=")([^"]*)(")')
CHART_KINDS = ("chart", "chartEx")
STYLE_KINDS = ("chartStyle", "chartColorStyle")
SHOWN_DIFFERENCES = 12


class Package:
    """A workbook's parts. A part is found whatever the case of its name, as the packaging rules compare part
    names (ECMA-376 Part 2): one input names a sheet `Sheet1.xml` and its relationships `sheet1.xml.rels`."""

    def __init__(self, path: Path) -> None:
        self.zip = zipfile.ZipFile(path)
        self.names = {name.lower(): name for name in self.zip.namelist()}

    def read(self, part: str) -> bytes | None:
        name = self.names.get(part.lower())
        return None if name is None else self.zip.read(name)

    def text(self, part: str) -> str:
        return (self.read(part) or b"").decode("utf-8", "replace")


def relationships(package: Package, part: str) -> dict[str, tuple[str, str]]:
    """The relationships of `part`: id -> (last word of the type, the target part's name)."""
    folder, name = posixpath.split(part)
    found = {}
    for element in RELATIONSHIP.findall(package.text(posixpath.join(folder, "_rels", name + ".rels"))):
        attributes = dict(ATTRIBUTE.findall(element))
        if attributes.get("TargetMode") == "External" or "Target" not in attributes:
            continue
        target = html.unescape(attributes["Target"])
        path = (
            target.lstrip("/")
            if target.startswith("/")
            else posixpath.normpath(posixpath.join(folder, target))
        )
        found[attributes.get("Id", "")] = (attributes.get("Type", "").rsplit("/", 1)[-1], path)
    return found


def charts(package: Package) -> dict[tuple[str, int], tuple[str, bytes]]:
    """Every chart of the workbook: (sheet name, its place among the sheet's charts) -> (chart part, frame)."""
    workbook = "xl/workbook.xml"
    targets = relationships(package, workbook)
    found = {}
    for element in SHEET.findall(package.text(workbook)):
        attributes = dict(ATTRIBUTE.findall(element))
        sheet = targets.get(next((v for k, v in attributes.items() if k.endswith(":id")), ""))
        if sheet is None:
            continue
        place = 0
        for kind, drawing in relationships(package, sheet[1]).values():
            if kind != "drawing":
                continue
            parts = relationships(package, drawing)
            for anchor in ANCHOR.finditer(package.text(drawing)):
                for reference in REFERENCE.findall(anchor.group(0)):
                    if parts.get(reference, ("", ""))[0] in CHART_KINDS:
                        place += 1
                        name = html.unescape(attributes.get("name", ""))
                        found[(name, place)] = (parts[reference][1], anchor.group(0).encode())
    return found


def same(before: bytes | None, after: bytes | None) -> str:
    if before is None:
        return "none" if after is None else "new"
    if after is None:
        return "gone"
    return "identical" if before == after else "differs"


def chart_verdict(before: bytes, after: bytes | None) -> str:
    """`same`, with one more answer: the two differ in series ids and nothing else."""
    answer = same(before, after)
    if answer != "differs" or after is None:
        return answer
    if SERIES_ID.sub(rb"\1\3", before) != SERIES_ID.sub(rb"\1\3", after):
        return "differs"
    old = [m[1] for m in SERIES_ID.findall(before)]
    new = [m[1] for m in SERIES_ID.findall(after)]
    changed = sum(1 for a, b in zip(old, new, strict=True) if a != b)
    return f"ids only ({changed} of {len(old)} changed)"


def related(package: Package, chart: str) -> dict[str, bytes]:
    """The parts a chart part relates to, by the last word of the relationship type (numbered if repeated)."""
    found: dict[str, bytes] = {}
    for kind, part in relationships(package, chart).values():
        data = package.read(part)
        if data is not None:
            key = kind if kind not in found else f"{kind} ({sum(1 for k in found if k.startswith(kind)) + 1})"
            found[key] = data
    return found


def differences(before: bytes, after: bytes) -> list[str]:
    """The first lines of a unified diff, with one element to a line."""
    lines = [
        re.sub(r">\s*<", ">\n<", data.decode("utf-8", "replace")).splitlines() for data in (before, after)
    ]
    diff = difflib.unified_diff(lines[0], lines[1], "before", "after", lineterm="", n=0)
    changed = [line for line in diff if line[:1] in "+-" and line[:3] not in ("+++", "---")]
    more = (
        [f"... and {len(changed) - SHOWN_DIFFERENCES} more lines"] if len(changed) > SHOWN_DIFFERENCES else []
    )
    return [line[:300] for line in changed[:SHOWN_DIFFERENCES]] + more


def compare(label: str, before_path: Path, after_path: Path) -> tuple[list[str], list[str]]:
    """The table's rows for one workbook, and the differences to list under the table."""
    rows, notes = [], []
    before, after = Package(before_path), Package(after_path)
    with before.zip, after.zip:
        old, new = charts(before), charts(after)
        for key in sorted(old):
            part, frame = old[key]
            data = before.read(part)
            if data is None:
                rows.append(f"| `{label}` | {key[0]} | {key[1]} | the input has no `{part}` | | | | |")
                continue
            saved = new.get(key)
            saved_data = after.read(saved[0]) if saved else None
            verdict = chart_verdict(data, saved_data)
            if verdict == "differs" and saved_data is not None:
                notes.append(f"`{label}`, {key[0]}, chart {key[1]} (`{part}`):")
                notes.extend(f"    {line}" for line in differences(data, saved_data))
            was = related(before, part)
            now = related(after, saved[0]) if saved else {}
            style, colours = (same(was.get(kind), now.get(kind)) for kind in STYLE_KINDS)
            others = sorted((set(was) | set(now)) - set(STYLE_KINDS))
            other = ", ".join(f"{kind} {same(was.get(kind), now.get(kind))}" for kind in others) or "none"
            frame_verdict = same(frame, saved[1] if saved else None)
            rows.append(
                f"| `{label}` | {key[0]} | {key[1]} | {verdict} | {style} | {colours} | {other} | {frame_verdict} |"
            )
        for key in sorted(set(new) - set(old)):
            rows.append(f"| `{label}` | {key[0]} | {key[1]} | new | | | | |")
        if not old and not new:
            rows.append(f"| `{label}` | no chart found | | | | | | |")
    return rows, notes


def pairs(before: Path, after: Path) -> list[tuple[str, Path, Path | None]]:
    if before.is_file():
        return [(before.name, before, after)]
    found = []
    for path in sorted(before.glob("*.xlsx")):
        saved = after / f"{path.stem}.web.xlsx"
        found.append((path.name, path, saved if saved.is_file() else None))
    return found


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    print("| Workbook | Sheet | Chart | Chart part | Style part | Colours part | Other parts | Frame |")
    print("|---|---|---|---|---|---|---|---|")
    notes = []
    for label, before, after in pairs(Path(sys.argv[1]), Path(sys.argv[2])):
        if after is None:
            print(f"| `{label}` | not returned | | | | | | |")
            continue
        try:
            rows, more = compare(label, before, after)
        except (zipfile.BadZipFile, KeyError, OSError) as error:
            rows, more = [f"| `{label}` | not readable: {type(error).__name__} | | | | | | |"], []
        print("\n".join(rows))
        notes.extend(more)
    if notes:
        print("\nWhere a chart part differs (one element to a line, `-` before the save, `+` after):\n")
        print("\n".join(notes))


if __name__ == "__main__":
    main()
