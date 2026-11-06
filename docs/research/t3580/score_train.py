"""T-3580 R4.1 train side: the engine and the correctly rounded value on cells.tsv's 3,435 F cdf/tail cells, the cells
T-3567 chose the r >= 1 form on (c5.py's filter). Engine values come from `rustsheet-eval show --all` of each case
($RUSTSHEET_EVAL or target/release/rustsheet-eval); the correctly rounded value is mpmath at 50 digits of the double
arguments, as fmath.py computes it. Run with the tools venv (mpmath):
  cd tools && uv run python ../docs/research/t3580/score_train.py"""
import csv, math, os, re, struct, subprocess
from pathlib import Path
import mpmath
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
EVAL = os.environ.get("RUSTSHEET_EVAL", str(ROOT / "target/release/rustsheet-eval"))


def train_rows():
    """(case, cell, x, m, n, upper, excel) of cells.tsv's F cdf/tail cells (c5.py's filter)."""
    rows = []
    for r in csv.DictReader(open(HERE.parent / "t2464/cells.tsv"), delimiter="\t"):
        if r["fn"] not in ("FDIST", "F.DIST.RT", "F.DIST"):
            continue
        p = r["args"].split("|")
        if r["fn"] == "F.DIST" and p[3] in ("False", "0.0"):
            continue
        try:
            x, m, n, e = float(p[0]), math.trunc(float(p[1])), math.trunc(float(p[2])), float(r["excel"])
        except ValueError:
            continue
        if x <= 0 or m < 1 or n < 1 or e == 0 or not math.isfinite(m * x):
            continue
        rows.append((r["case"], r["cell"], x, m, n, r["fn"] != "F.DIST", e))
    return rows


def engine_values(cases):
    """{(case, Sheet!A1): the engine's displayed value} from `rustsheet-eval show --all` of each case."""
    actual = {}
    for case in sorted(cases):
        out = subprocess.run([EVAL, "show", "--all", case], capture_output=True, text=True, cwd=ROOT).stdout
        cur = None
        for line in out.splitlines():
            hit = re.match(r"(\S+!\S+)\s+\S+", line)
            if hit and not line.startswith(" "):
                cur = hit.group(1)
            hit = re.match(r"\s+actual\s*: (.*)", line)
            if hit and cur:
                actual[(case, cur)] = hit.group(1)
    return actual


def correctly_rounded(x, m, n, upper):
    """The F tail (upper) or cdf at the double x and integer dfs m, n, mpmath at 50 digits, as a double."""
    mpmath.mp.dps = 50
    xm, d1, d2 = mpmath.mpf(x), mpmath.mpf(m), mpmath.mpf(n)
    if upper:
        return float(mpmath.betainc(d2 / 2, d1 / 2, 0, d2 / (d1 * xm + d2), regularized=True))
    return float(mpmath.betainc(d1 / 2, d2 / 2, 0, d1 * xm / (d1 * xm + d2), regularized=True))


def ordered(v):
    b = struct.unpack("<q", struct.pack("<d", v))[0]
    return b if b >= 0 else -(b & 0x7FFFFFFFFFFFFFFF)
def tally(pairs):
    ex = w4 = w15 = 0
    for a, e in pairs:
        ex += a == e
        w4 += math.isfinite(a) and abs(ordered(a) - ordered(e)) <= 4
        w15 += abs(a - e) <= 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14)
    return ex, w4, w15


def main():
    rows = train_rows()
    actual = engine_values({row[0] for row in rows})
    eng, cr = [], []
    for case, cell, x, m, n, upper, e in rows:
        try:
            a = float(actual[(case, cell)])
        except (KeyError, ValueError):
            a = math.nan
        eng.append((a, e))
        cr.append((correctly_rounded(x, m, n, upper), e))
    print(sum(math.isnan(a) for a, _ in eng), "without an engine value;", len(rows), "cells; engine exact/w4/w15", tally(eng), "; correctly rounded", tally(cr))


if __name__ == "__main__":
    main()
