"""T-4497: score the a = 1 tail forms on the sweep (docs/research/t4497/README.md).

Reads `rustsheet-eval show --all` output of each case under each form, one file per form named
<form>.<case>.show (made with a scratch build whose `unoriented_shape_one_tail` reads the form from
T4497_MODE: libm = -expm1(b ln1p(-x)) by libm, cancel = 1 - exp(b ln(1 - x)), excel = Excel's
1 - e^x of b times its ln(1 - x) (x87_exp / x87_log1p), unset = today's 0.1 cut), and points.tsv.
A cut form c takes the below form where x < c and `cancel` from c up.

Usage (repo root): python3 docs/research/t4497/score.py SHOW_DIR
"""

from __future__ import annotations

import math
import re
import struct
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
FORMS = ("today", "libm", "cancel", "excel")


def ordinal(v: float) -> int:
    i = struct.unpack("<q", struct.pack("<d", v))[0]
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def grade(e: float, a: float) -> str:
    if e == a:
        return "bit"
    if abs(ordinal(e) - ordinal(a)) <= 4:
        return "ulp4"
    if f"{e:.15g}" == f"{a:.15g}":
        return "d15"
    return "off"


def parse(path: Path) -> dict[str, tuple[float, float]]:
    out: dict[str, tuple[float, float]] = {}
    cell = None
    exp = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if m := re.match(r"^Calc!(\w+)\s", line):
            cell = m.group(1)
        elif m := re.match(r"^\s+expected: (\S+)", line):
            exp = float(m.group(1))
        elif (m := re.match(r"^\s+actual\s*: (\S+)", line)) and cell and exp is not None:
            out[cell] = (exp, float(m.group(1)))
            exp = None
    return out


def main() -> None:
    show = Path(sys.argv[1])
    points = []
    with (HERE / "points.tsv").open(encoding="utf-8") as f:
        next(f)
        for line in f:
            case, cell, fn, x, b, _math, hold = line.rstrip("\n").split("\t")
            points.append((case, cell, fn, float(x), float(b), hold == "1"))
    vals = {form: {} for form in FORMS}
    for form in FORMS:
        for case in sorted({p[0] for p in points}):
            for cell, ev in parse(show / f"{form}.{case}.show").items():
                vals[form][(case, cell)] = ev

    def tally(pick, subset):
        counts = {"bit": 0, "ulp4": 0, "d15": 0, "off": 0}
        for p in subset:
            e, a = vals[pick(p)][(p[0], p[1])]
            counts[grade(e, a)] += 1
        n = len(subset)
        return f"{counts['bit']}/{n} bit, {counts['bit'] + counts['ulp4']} <=4ulp, {n - counts['off']} 15d, {counts['off']} off"

    train = [p for p in points if not p[5]]
    hold = [p for p in points if p[5]]
    # Per-cell: which forms are bit-exact, by x.
    print("## Per-point bit-exact forms (x ascending)")
    for p in sorted(points, key=lambda p: p[3]):
        hit = [f for f in ("libm", "cancel", "excel") if vals[f][(p[0], p[1])][0] == vals[f][(p[0], p[1])][1]]
        print(f"{p[3]:.6g}\t{p[2]}\t{p[0][-5:]}!{p[1]}\tb={p[4]:g}\t{'H' if p[5] else 'T'}\t{','.join(hit) or '-'}")
    print()
    print("## Uniform forms (training / hold-out / all)")
    for form in FORMS:
        print(f"{form}: {tally(lambda p, f=form: f, train)} | {tally(lambda p, f=form: f, hold)} | {tally(lambda p, f=form: f, points)}")
    print()
    print("## By function at the 0.1 cut: today (libm below) / Excel kernels below (training | hold-out)")
    for fn in ("F.TEST", "F.INV", "F.INV.RT"):
        tr = [p for p in train if p[2] == fn]
        ho = [p for p in hold if p[2] == fn]
        for below in ("libm", "excel"):
            pick = lambda p, below=below: below if p[3] < 0.1 else "cancel"
            print(f"{fn} {below}: {tally(pick, tr)} | {tally(pick, ho)}")
    print()
    print("## Visibly off today")
    for p in points:
        e, a = vals["today"][(p[0], p[1])]
        if grade(e, a) == "off":
            print(f"{p[0]}!{p[1]}\t{p[2]}\tx={p[3]:.6g}\tb={p[4]:g}\texpected {e!r}\tactual {a!r}")
    print()
    print("## Cut scan on training cells: below-form where x < c, cancel from c up")
    xs = sorted({p[3] for p in train})
    cands = [xs[0] / 2] + [math.sqrt(xs[i] * xs[i + 1]) for i in range(len(xs) - 1)] + [xs[-1] * 2]
    for below in ("libm", "excel"):
        best = None
        for c in cands:
            n = sum(1 for p in train if vals[below if p[3] < c else "cancel"][(p[0], p[1])][0] == vals[below if p[3] < c else "cancel"][(p[0], p[1])][1])
            if best is None or n > best[0]:
                best = (n, c, c)
            elif n == best[0]:
                best = (n, best[1], c)
        n, lo, hi = best
        print(f"{below} below: best training bit-exact {n}/{len(train)} for c in [{lo:.6g}, {hi:.6g}]")
        for c in (0.1, lo, hi):
            pick = lambda p, c=c, below=below: below if p[3] < c else "cancel"
            print(f"  c={c:.6g}: train {tally(pick, train)} | hold {tally(pick, hold)} | all {tally(pick, points)}")


if __name__ == "__main__":
    main()
