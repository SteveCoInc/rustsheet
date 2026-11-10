#!/usr/bin/env python3
"""Write the unfilled val4 NSI probe request (T-5112, round 8): long runs of consecutive doubles x
with the single readout GAMMA.INV(x,0.5,1), whose binary64 square root is Excel's hidden central
inverse-erf core w (val3-nsi-01: 192/192, val3-nsi-02: 9/9, val2-nsi-01: 12/12).

    python3 next_val4_nsi.py OUTPUT_DIR

Seven `rustsheet-case/v1` files, no expected values, every formula cell in `check`. Every x is
written as an exact dyadic expression `(hi*67108864+lo)/2^k` (integer limbs below 2^53, a power-of-two
divisor), so no 15-digit literal truncation can occur. Nothing here reads oracle values.
"""
from __future__ import annotations

import json
import math
import struct
import sys
from pathlib import Path

RUN = 1024
LIMB = 67108864  # 2^26


def bits(x: float) -> int:
    return struct.unpack("<q", struct.pack("<d", x))[0]


def from_bits(b: int) -> float:
    return struct.unpack("<d", struct.pack("<q", b))[0]


def dyadic(x: float) -> str:
    """`(hi*67108864+lo)/2^k`, exactly x, for a positive normal double below 1."""
    mantissa, exponent = math.frexp(x)  # x = mantissa * 2^exponent, mantissa in [0.5, 1)
    m = int(mantissa * (1 << 53))
    k = 53 - exponent
    hi, lo = divmod(m, LIMB)
    assert (hi * LIMB + lo) / 2.0**k == x and hi < (1 << 27)
    return f"({hi}*{LIMB}+{lo})/2^{k}"


def consecutive(x0: float, count: int) -> list[float]:
    b = bits(x0)
    return [from_bits(b + i) for i in range(count)]


def a_run_starts() -> list[float]:
    """64 irregular starts, uniform in t = x*x over (0.083, 0.494): piece A, both sides of x = 0.5."""
    return [math.sqrt(0.08 + 0.0065137 * i + 0.0031415) for i in range(64)]


def b_run_starts() -> list[float]:
    """16 irregular starts in the second piece, fl(sqrt(.5)) < x < 0.85."""
    return [0.7080 + 0.0088137 * j + 0.0003141 for j in range(16)]


SQRT_HALF = 0.7071067811865476  # fl(sqrt(0.5)), 0x3FE6A09E667F3BCD: the first double of piece B


def below(x_end: float, count: int) -> list[float]:
    """The `count` doubles just below x_end (x_end excluded)."""
    b = bits(x_end)
    return [from_bits(b - count + i) for i in range(count)]


def epsilon_ladders() -> list[tuple[str, list[float]]]:
    out = []
    for name, t_anchor in (("t=1/2", 0.5), ("t=1/4", 0.25)):
        for e in range(4, 32, 2):
            x0 = math.sqrt(t_anchor * (1.0 - 2.0**-e))
            xs = consecutive(x0, 256)
            assert xs[-1] < math.sqrt(t_anchor)
            out.append((f"{name} eps=2^-{e}", xs))
    return out


def scan_points() -> list[tuple[str, list[float]]]:
    out = []
    for k in range(29):  # 0.71 .. 0.99
        x = 0.71 + 0.01 * k
        out.append((f"scan {x:.2f}", consecutive(x, 16)))
    for k in range(7, 21):
        x = 1.0 - 2.0**-k
        out.append((f"scan 1-2^-{k}", consecutive(from_bits(bits(x) - 8), 16)))
    for name, x in (("0.85", 0.85), ("sqrt(0.75)", math.sqrt(0.75)), ("0.75", 0.75), ("0.9375", 0.9375)):
        out.append((f"window {name}", consecutive(from_bits(bits(x) - 32), 64)))
    return out


PRECISION = {
    "name": "Precision",
    "cells": {
        "A1": "offset", "B1": "raw", "C1": "raw subtraction, not SUM",
        "A2": -1, "B2": "=1+A2*2^-52", "C2": "=(B2-1)*2^52",
        "A3": 0, "B3": "=1+A3*2^-52", "C3": "=(B3-1)*2^52",
        "A4": 1, "B4": "=1+A4*2^-52", "C4": "=(B4-1)*2^52",
    },
}
PRECISION_CHECK = [f"Precision!{c}{r}" for r in (2, 3, 4) for c in ("B", "C")]
TAIL = (
    " Readout: w = SQRT(cell) offline (binary64); no expected values; score raw bits."
    " Precision!B2:B4 must retain 0.9999999999999998, 1, 1.0000000000000002; C2:C4 must be -1,0,1."
    " Predictions (hypotheses, NOT oracle values): docs/research/val4-nsi-predictions.json;"
    " protocol: docs/research/probe-handoff-2026-11-10-val4-nsi.md."
)


def gamma(x: float) -> str:
    return f"=GAMMA.INV({dyadic(x)},0.5,1)"


def case(description: str, sheets: list[dict]) -> dict:
    check = []
    for sheet in sheets:
        for address, value in sheet["cells"].items():
            if isinstance(value, str) and value.startswith("="):
                check.append(f"{sheet['name']}!{address}")
    return {
        "format": "rustsheet-case/v1",
        "description": description + TAIL,
        "tags": ["M9", "statistical", "probe"],
        "sheets": sheets + [PRECISION],
        "check": check + PRECISION_CHECK,
    }


def column(xs: list[float], col: str = "A", first: int = 1) -> dict[str, str]:
    return {f"{col}{first + i}": gamma(x) for i, x in enumerate(xs)}


def build() -> dict[str, dict]:
    cases: dict[str, dict] = {}
    starts = a_run_starts()
    for part in range(4):
        cells: dict[str, str] = {}
        for local, x0 in enumerate(starts[16 * part : 16 * part + 16]):
            cells.update(column(consecutive(x0, RUN), "A", 1 + RUN * local))
        cases[f"val4-nsi-0{part + 1}"] = case(
            f"NSI val4-0{part + 1}: piece-A runs {16 * part}..{16 * part + 15} of 64. R!A holds 16 runs of 1024"
            " consecutive doubles x (run r occupies rows 1024*r+1..1024*r+1024), x0 = fl(sqrt(0.08 +"
            " 0.0065137*i + 0.0031415)), i the global run index; each cell is GAMMA.INV(x,0.5,1).",
            [{"name": "R", "cells": cells}],
        )
    cells = column(below(SQRT_HALF, 8192), "A", 1)
    cells.update(column(below(0.5, 8192), "A", 8193))
    cases["val4-nsi-05"] = case(
        "NSI val4-05: runs below powers of two of t = x*x. R!A1:A8192 are the 8192 doubles just below"
        " fl(sqrt(0.5)) = 0.7071067811865476 (t -> 1/2 from below, the end of piece A); R!A8193:A16384 are the"
        " 8192 doubles just below 0.5 (t -> 1/4 from below). Each cell is GAMMA.INV(x,0.5,1).",
        [{"name": "R", "cells": cells}],
    )
    cells = {}
    controls: dict[str, str] = {}
    row = 1
    for index, (_, xs) in enumerate(epsilon_ladders()):
        cells.update(column(xs, "A", row))
        if index < 8:
            for i, x in enumerate(xs[:64]):
                controls[f"B{row + i}"] = f"=-NORM.S.INV((1-{dyadic(x)})/2)"
        row += len(xs)
    scan: dict[str, str] = {}
    row = 1
    for _, xs in scan_points():
        scan.update(column(xs, "A", row))
        row += len(xs)
    cells.update(controls)
    cases["val4-nsi-06"] = case(
        "NSI val4-06: epsilon ladders, z controls and the piece map. E!A: 28 ladders of 256 consecutive"
        " doubles starting at fl(sqrt(T*(1-2^-e))), T = 1/2 then 1/4, e = 4,6,..,30 (ladder k occupies rows"
        " 256*k+1..256*k+256). E!B: -NORM.S.INV((1-x)/2) on the first 64 rows of the first 8 ladders (the"
        " z = RN53(RN64(sqrt2*w)) control). S!A: 16 consecutive doubles at x = 0.71,0.72,..,0.99 (rows 1..464),"
        " 16 around 1-2^-k for k = 7..20 (rows 465..688), then 64-double windows centred on 0.85, sqrt(0.75),"
        " 0.75 and 0.9375 (rows 689..944): where the inverse-erf core changes piece above sqrt(0.5).",
        [{"name": "E", "cells": cells}, {"name": "S", "cells": scan}],
    )
    cells = {}
    for local, x0 in enumerate(b_run_starts()):
        cells.update(column(consecutive(x0, RUN), "A", 1 + RUN * local))
    cases["val4-nsi-07"] = case(
        "NSI val4-07: piece-B runs. R!A holds 16 runs of 1024 consecutive doubles (run j occupies rows"
        " 1024*j+1..1024*j+1024), x0 = 0.7080 + 0.0088137*j + 0.0003141, all inside sqrt(0.5) < x < 0.85;"
        " each cell is GAMMA.INV(x,0.5,1).",
        [{"name": "R", "cells": cells}],
    )
    return cases


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    total = 0
    for name, body in build().items():
        if "expected" in body:
            raise SystemExit("a probe request never carries expected values")
        (out / f"{name}.case.json").write_text(json.dumps(body, indent=1) + "\n")
        formulas = len(body["check"])
        total += formulas
        print(f"{name}: {formulas} checked formula cells")
    print(f"total: {total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
