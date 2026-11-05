"""T-3413 attempt 1: score CUMPRINC principal forms through Excel's CUMIPMT = -pmt*m - CUMPRINC.

Run: python3 docs/research/t3413/fit.py (fit2.py scores orderings of the geometric sum).
Each printed list is the interest's distance from Excel in ulps of the interest, one entry per cell of
cells.json (the T-3413 visibly-off CUMIPMT cells of fin-cumipmt-gate-edges and
fin-cumipmt-tiny-rate-structure). Python's math (glibc) stands in for Excel's exp/expm1/log1p.
The `balance*` forms are type 0 only and are not scored on type 1 cells.
"""

import json
import struct
from collections.abc import Callable, Iterator
from math import exp, expm1, log1p
from pathlib import Path

Cell = tuple[float, float, float, int, int, int, float]


def load_cells() -> list[Cell]:
    """The scored cells: rate, nper, pv, start, end, type and Excel's CUMIPMT."""
    rows = json.loads((Path(__file__).parent / "cells.json").read_text())
    keys = ("rate", "nper", "pv", "start", "end", "type", "excel_cumipmt")
    return [tuple(row[k] for k in keys) for row in rows]  # type: ignore[misc]


def pmts(r: float, n: float, pv: float, t: int) -> dict[str, float]:
    """Candidate payment forms, `d = 1 - v^nper = -expm1(-nper*L)`."""
    d = -expm1(-n * log1p(r))
    if t == 0:
        return {"(pv/d)r": (pv / d) * r, "pv r/d": pv * r / d}
    return {
        "(pv/d)r/(1+r)": (pv / d) * r / (1 + r),
        "pv r/d/(1+r)": pv * r / d / (1 + r),
        "pv(r/(1+r))/d": pv * (r / (1 + r)) / d,
    }


def princ_forms(r: float, n: float, pv: float, s: int, e: int, t: int, p: float) -> dict[str, float]:
    """Candidate principal magnitudes (CUMPRINC is minus these)."""
    big_l = log1p(r)
    m = e - s + 1
    first = s if t == 0 else max(s, 2)
    out = {
        "perperiod": sum(p * exp((k - 1 - n) * big_l) for k in range(first, e + 1)),
        "geom": (p * ((-expm1(-(e - first + 1) * big_l)) / r * (1 + r))) * exp(-(n - e + 1) * big_l),
        "closed": p * ((exp((e - n) * big_l) - exp((s - 1 - n) * big_l)) / expm1(big_l)),
        "closed2": p * exp((s - 1 - n) * big_l) * expm1(m * big_l) / expm1(big_l),
    }
    if t == 0:
        q1, q2 = exp((s - 1) * big_l), exp(e * big_l)
        out["balance"] = (pv * q1 - p * (q1 - 1) / r) - (pv * q2 - p * (q2 - 1) / r)
        out["balance_fact"] = (pv - p / r) * (expm1((s - 1) * big_l) - expm1(e * big_l))
        out["balance_fact2"] = (p / r - pv) * exp((s - 1) * big_l) * expm1(m * big_l)
        out["balance_fact3"] = (p / r - pv) * (exp(e * big_l) - exp((s - 1) * big_l))
    return out


def ordered(x: float) -> int:
    """The double's position on the number line, so that a difference counts ulps."""
    i = struct.unpack("<q", struct.pack("<d", x))[0]
    return -(i & 0x7FFFFFFFFFFFFFFF) if i < 0 else i


Forms = Callable[[float, float, float, int, int, int, float], dict[str, float]]


def scores(forms: Forms) -> Iterator[tuple[tuple[int, str, str], list[int]]]:
    """Per (type, payment form, principal form), the interest's ulp distance from Excel per cell."""
    table: dict[tuple[int, str, str], list[int]] = {}
    for r, n, pv, s, e, t, excel in load_cells():
        m = e - s + 1
        for pname, p in pmts(r, n, pv, t).items():
            for fname, x in forms(r, n, pv, s, e, t, p).items():
                interest = -p * m + x
                table.setdefault((t, pname, fname), []).append(ordered(interest) - ordered(excel))
    yield from sorted(table.items(), key=lambda kv: (kv[0][0], -kv[1].count(0)))


def main(forms: Forms, at_least: int = 0) -> None:
    """Print each form's bit-exact count, cell count and ulp list."""
    for key, dist in scores(forms):
        if dist.count(0) >= at_least:
            print(dist.count(0), len(dist), key, dist)


if __name__ == "__main__":
    main(princ_forms)
