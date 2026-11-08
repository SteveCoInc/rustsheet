"""Build probes/t4466-count-hypotheses: python3 gen.py CASE_PATH (writes meta.json beside this script).

T-4466 / T-4333: does Excel's elapsed count before a fraction of a second depend on the fraction digits shown, and
does `[s]` alone flip where `[s].0` does? Rows 1-40 are the eight tie serials of probes/t4333-count-ulp-scan
(docs/research/t4333/ulp_scan.py) stepped -2..+2 of their own ulp; columns are the formats in FORMATS. Rows 41-80 are
the same seeds moved to their `.00` tie (k.995 s) and rows 81-120 to their `.000` tie (k.9995 s), each under `[s].00`
and `[s].000`. Rows 121-152 are a hold-out drawn with random.Random(4466), fixed before any oracle value was read:
32 fresh `d + k.95` ties at the nearest double, under `[s].0`, `[s].0 General` and `[m]:ss.0`.
`meta.json` maps each row to [day, seconds text, ulp offset, ulp exponent].
"""

import json
import math
import os
import random
import sys

SEEDS = [
    (45000, "0.95"),
    (3, "5.95"),
    (1, "217.95"),
    (45000, "59.95"),
    (45000, "2740.95"),
    (0, "3733.95"),
    (60000, "82799.95"),
    (1, "59.95"),
]
OFFSETS = range(-2, 3)
FORMATS = ["[s]", "[s].0", "[s].00", "[s].000", "[m]:ss.0", "[h]:mm:ss.0", "[s].0 General"]
COLS = "ABCDEFG"


def serial(d, k):
    return (d + float(k) / 86400) if d else float(k) / 86400


def expr(d, k, off, e):
    base = f"{d}+{k}/86400" if d else f"{k}/86400"
    return base + ("" if off == 0 else f"{off:+d}*2^{e}")


cells, meta = {}, {}
row = 0
for d, k in SEEDS:
    for off in OFFSETS:
        row += 1
        e = round(math.log2(math.ulp(serial(d, k))))
        for col, f in zip(COLS, FORMATS):
            cells[f"{col}{row}"] = f'=TEXT({expr(d, k, off, e)},"{f}")'
        meta[str(row)] = [d, k, off, e]
for tail, fmts in (("995", ["[s].00", "[s].000"]), ("9995", ["[s].00", "[s].000"])):
    for d, k in SEEDS:
        kk = k.split(".")[0] + "." + tail
        for off in OFFSETS:
            row += 1
            e = round(math.log2(math.ulp(serial(d, kk))))
            for col, f in zip(COLS, fmts):
                cells[f"{col}{row}"] = f'=TEXT({expr(d, kk, off, e)},"{f}")'
            meta[str(row)] = [d, kk, off, e]
rng = random.Random(4466)
DAYS = [0, 1, 2, 3, 5, 7, 30, 365, 20000, 45000, 60000, 2958000]
for _ in range(32):
    row += 1
    d = rng.choice(DAYS)
    k = f"{rng.randrange(0, 86400 if d else 86399)}.95"
    for col, f in zip(COLS, ["[s].0", "[s].0 General", "[m]:ss.0"]):
        cells[f"{col}{row}"] = f'=TEXT({expr(d, k, 0, 0)},"{f}")'
    meta[str(row)] = [d, k, 0, 0]
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4466: the elapsed count before a fraction at the t4333 ulp-scan tie serials, under [s], "
    "[s].0/.00/.000, [m]:ss.0, [h]:mm:ss.0 and [s].0 General, the .00/.000 ties of the same "
    "seeds, and a fixed-seed hold-out (docs/research/t4466/gen.py).",
    "tags": ["M10", "text", "probe"],
    "sheets": [{"name": "Sheet1", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as fh:
    fh.write(json.dumps(case, indent=2) + "\n")
with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "meta.json"), "w") as fh:
    json.dump(meta, fh, indent=1)
print(len(cells), "cells")
