"""T-3771 probe t3771-r6-bpser-gpin-ln: Excel's =LN(x) at every x of t3570-r5-bpser-lg (short decimals, written as
typed), so a bpser miss can be split between LN and the rest. Meta: [[row, x], ...].
usage: mkprobe.py [--write]"""
from __future__ import annotations

import json
import sys

import gpin

pin, lib = gpin.pin, gpin.lib
NAME = "t3771-r6-bpser-gpin-ln"


def main() -> None:
    xs = sorted({x for _, _, _, xx in json.load(open(pin.HERE / "t3570-r5-bpser-lg.meta.json")) for x in xx})
    rows = [(i, x) for i, x in enumerate(xs, 1)]
    cells = {f"A{i}": f"=LN({lib.w2p.num(x)})" for i, x in rows}
    print(NAME, len(cells), "cells")
    if "--write" in sys.argv:
        case = {
            "format": "rustsheet-case/v1",
            "description": "T-3771 r6-bpser-gpin: LN at every x of t3570-r5-bpser-lg (docs/research/t2464/r6-bpser-gpin)",
            "tags": ["M9", "probe", "statistics", "T-3771", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}],
            "expected": {},
        }
        (lib.CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        json.dump(rows, open(gpin.HERE / f"{NAME}.meta.json", "w"))


if __name__ == "__main__":
    main()
