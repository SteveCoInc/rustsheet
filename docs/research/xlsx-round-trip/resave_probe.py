"""T-4890: compare Excel for the web's re-saves of our saves with its re-saves of the sources.

Layout under DIR (see excel-resave-probe-2026-11-07.md):
  src/, csrc/            the sources (fixtures; the corpus sample), named `<case id with / as __>`
  src-excel/, csrc-excel/ `rustsheet-oracle resave` of them
  ours/                  our saves from `rustsheet-eval roundtrip --save-dir`, `<source>.<scenario>.xlsx`
  ours-excel/            `rustsheet-oracle resave` of them

Usage: python3 -I resave_probe.py DIR path/to/rustsheet-eval
Prints, per scenario, saves Excel opened or refused and whether each was patched (workbook.xml kept
byte for byte) or regenerated; then each R0 save whose Excel re-save differs from Excel's re-save of
its source in anything but the SharePoint save's own noise.
"""

from __future__ import annotations

import collections
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

# Written by every SharePoint save whatever the input: ids, dates, version, and the customXml
# parts it adds and numbers in no fixed order.
NOISE = re.compile(
    r"customXml|revisionPtr|dcterms:(created|modified)|ep:AppVersion|cp:lastModifiedBy|xr:uid|datastoreItem"
)


def source_of(save: Path, root: Path) -> Path:
    base = save.name[: -len(".xlsx")].rsplit(".", 1)[0]
    return root / ("src" if base.startswith("roundtrip-fixtures") else "csrc") / base


def package_diff(exe: str, a: Path, b: Path) -> list[dict[str, object]]:
    run = subprocess.run([exe, "package-diff", str(a), str(b), "--json"], capture_output=True, text=True)
    if run.returncode != 0:
        raise SystemExit(f"package-diff {a} {b}: {run.stderr.strip()}")
    # The importer still prints some diagnostics on stdout (T-4910): the JSON is the last line.
    return list(json.loads(run.stdout.strip().splitlines()[-1]))


def col(ref: str) -> int:
    n = 0
    for ch in re.match(r"[A-Z]+", ref).group():  # type: ignore[union-attr]
        n = n * 26 + ord(ch) - 64
    return n


def hints(src: zipfile.ZipFile, ours: zipfile.ZipFile) -> set[str]:
    """Which of the patched-package hints (xlsx-storage.md) a patched save exercises."""
    a, b = set(src.namelist()), set(ours.namelist())
    found = set()
    for n in sorted(a & b):
        if not n.startswith("xl/worksheets/sheet") or src.read(n) == ours.read(n):
            continue
        found.add("rewrote a worksheet")
        x, y = src.read(n).decode("utf8", "replace"), ours.read(n).decode("utf8", "replace")
        dim = re.compile(r'<dimension ref="([^"]+)"')
        if (m := dim.search(x)) and (k := dim.search(y)) and m[1] != k[1]:
            found.add("changed a dimension")
        for row in re.finditer(r'<row [^>]*spans="(\d+):(\d+)"[^>]*>(.*?)</row>', y):
            if any(not int(row[1]) <= col(c) <= int(row[2]) for c in re.findall(r'<c r="([A-Z]+)\d+"', row[3])):
                found.add("a cell outside its row's spans")
    if "xl/calcChain.xml" in a:
        found.add("copied a calcChain" if "xl/calcChain.xml" in b else "dropped a calcChain")
    if "xl/metadata.xml" in b - a:
        found.add("added the metadata parts")
    for part, what in [("xl/styles.xml", "appended styles"), ("xl/sharedStrings.xml", "appended shared strings")]:
        if part in a & b and src.read(part) != ours.read(part):
            found.add(what)
    return found


def main() -> None:
    root, exe = Path(sys.argv[1]), sys.argv[2]
    status = json.loads((root / "ours-excel/resave.json").read_text())
    tally: collections.Counter[tuple[str, str, str]] = collections.Counter()
    exercised: collections.Counter[str] = collections.Counter()
    for save in sorted((root / "ours").iterdir()):
        label = save.name[: -len(".xlsx")].rsplit(".", 1)[1]
        src = zipfile.ZipFile(source_of(save, root))
        ours = zipfile.ZipFile(save)
        patched = "xl/workbook.xml" in src.namelist() and ours.read("xl/workbook.xml") == src.read(
            "xl/workbook.xml"
        )
        opened = status[save.name]["status"]
        tally[(label, "patched" if patched else "regenerated", opened)] += 1
        if patched and opened == "saved":
            exercised.update(hints(src, ours))
    for key, n in sorted(tally.items()):
        print(n, *key)
    for hint, n in sorted(exercised.items()):
        print(f"{n} opened patched saves {hint}")
    for save in sorted((root / "ours").glob("*.R0.xlsx")):
        src = source_of(save, root)
        real = [
            i
            for i in package_diff(
                exe,
                src.parent.with_name(src.parent.name + "-excel") / src.name,
                root / "ours-excel" / save.name,
            )
            if not NOISE.search(str(i["key"]))
        ]
        if real:
            print("R0 differs after Excel's save:", save.name, [i["key"] for i in real][:3])


if __name__ == "__main__":
    main()
