"""Count which Excel functions real-world workbooks call (decision 0355 usage tiers).

Reads every `*_input.xlsx` under the SpreadsheetBench corpus that corpus-bench fetches
(eval/suites/corpus-bench/suite.toml, `[fetch]`: the tarball and its sha256), and prints, for each
function, how many workbooks call it and in how many formula cells. A shared formula counts once per
row of its range. Prefixes Excel writes for newer functions (`_xlfn.`, `_xlws.`, `_xleta.`) are
dropped, so `_xlfn._xlws.FILTER` counts as FILTER.

Usage: python3 -I count_usage.py <extracted corpus dir> > counts.txt  (writes usage.json beside cwd)
"""

import collections
import glob
import html
import json
import re
import sys
import zipfile

FORMULA = re.compile(rb"<(?:\w+:)?f\b([^>]*)>(.*?)</(?:\w+:)?f>", re.S)
CALL = re.compile(r"(?<![A-Za-z0-9_.])((?:_xlfn\.|_xlws\.|_xleta\.)*[A-Z][A-Z0-9.]*)\(")
SHARED_RANGE = re.compile(rb'ref="([A-Z]+)(\d+):([A-Z]+)(\d+)"')
SHEET = re.compile(r"xl/worksheets/sheet\d+\.xml$")
# A sheet larger than this uncompressed is skipped: the corpus is pinned, but the cap keeps a
# hostile archive from exhausting memory if the script is ever pointed elsewhere.
MAX_SHEET_BYTES = 256 * 1024 * 1024


def bare(name: str) -> str:
    return name.replace("_xlfn.", "").replace("_xlws.", "").replace("_xleta.", "")


def main(root: str) -> None:
    cells: collections.Counter[str] = collections.Counter()
    books: collections.Counter[str] = collections.Counter()
    with_formulas = 0
    files = sorted(glob.glob(f"{root}/**/*_input.xlsx", recursive=True))
    for path in files:
        try:
            archive = zipfile.ZipFile(path)
        except (OSError, zipfile.BadZipFile):
            continue
        seen: set[str] = set()
        any_formula = False
        for member in archive.namelist():
            if not SHEET.match(member) or archive.getinfo(member).file_size > MAX_SHEET_BYTES:
                continue
            try:
                data = archive.read(member)
            except (OSError, zipfile.BadZipFile):
                continue
            for attrs, body in FORMULA.findall(data):
                text = html.unescape(body.decode("utf8", "replace"))
                if not text:
                    continue
                any_formula = True
                weight = 1
                shared = SHARED_RANGE.search(attrs)
                if b't="shared"' in attrs and shared:
                    weight = max(1, int(shared.group(4)) - int(shared.group(2)) + 1)
                for name in {bare(n) for n in CALL.findall(text)}:
                    cells[name] += weight
                    seen.add(name)
        with_formulas += any_formula
        books.update(seen)
    summary = {"files": len(files), "workbooks_with_formulas": with_formulas, "cells": cells, "books": books}
    with open("usage.json", "w", encoding="utf8") as out:
        json.dump(summary, out)
    print(f"{len(files)} files, {with_formulas} with formulas")
    for name, count in books.most_common():
        print(f"{name}\t{count}\t{cells[name]}")


if __name__ == "__main__":
    main(sys.argv[1])
