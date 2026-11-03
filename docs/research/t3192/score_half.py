"""T-3192: score a = b = 1/2 closed forms on Excel's cells.

Sweep 1: probes-numeric/t3192-beta-half-half (column B BETA.DIST; C BETADIST equals it on every row).
Sweep 2: probes-numeric/t3192-beta-half-half-2 (column B), filled after the 0.9 switch was frozen on sweep 1.
`Er` was the choice frozen on the 49 cells on file before T-3192 (reflection at x > 1/2); sweep 1 rejected it and
fixed the switch at 0.9 (`switch 0.9`, the ported form, `incomplete_beta::half_half_cdf`), which sweep 2 tests.
Usage (repo root): tools/.venv/bin/python docs/research/t3192/score_half.py
"""
import json
import math
from pathlib import Path

from mpmath import asin, atan, betainc, mp, mpf

mp.dps = 40
CASE = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half")
CASE2 = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half-2")
sq = math.sqrt


def refl(f):
    return lambda x: f(x) if x <= 0.5 else (0.5 - f(1 - x)) + 0.5


FORMS = {
    "switch 0.9 (ported)": lambda x: (
        2 * math.atan(sq(x) / sq(1 - x)) / math.pi
        if x <= 0.9
        else (0.5 - 2 * math.atan(sq(1 - x) / sq(x)) / math.pi) + 0.5
    ),
    "Er (frozen)": refl(lambda x: 2 * math.atan(sq(x) / sq(1 - x)) / math.pi),
    "E direct": lambda x: 2 * math.atan(sq(x) / sq(1 - x)) / math.pi,
    "A asin direct": lambda x: 2 * math.asin(sq(x)) / math.pi,
    "A asin reflected": refl(lambda x: 2 * math.asin(sq(x)) / math.pi),
    "asin(fl sqrt)/fl(pi) reflected": refl(lambda x: float(2 * asin(mpf(sq(x))) / mpf(math.pi))),
    "correctly rounded": lambda x: float(betainc(mpf(0.5), mpf(0.5), 0, mpf(x), regularized=True)),
}


def load(case_path):
    case = json.loads(case_path.with_suffix(".case.json").read_text())["sheets"][0]["cells"]
    oracle = json.loads(case_path.with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
    rows = []
    i = 1
    while f"A{i}" in case:
        row = [oracle.get(f"{col}{i}", {}).get("v") for col in "BCD"]
        rows.append((case[f"A{i}"], *row))
        i += 1
    return rows


def main():
    rows = load(CASE)
    print(f"{len(rows)} rows; BETA.DIST == BETADIST on {sum(b == c for _, b, c, _ in rows)}; "
          f"BETA.DIST == worksheet 2*ATAN(..)/PI() on {sum(b == d for _, b, _, d in rows)}")
    score(rows)
    print("sweep 2")
    score(load(CASE2))


def score(rows):
    for name, f in FORMS.items():
        ok = [f(x) == b for x, b, _, _ in rows]
        within = sum(abs(f(x) - b) <= 4 * math.ulp(b) for x, b, _, _ in rows)
        print(f"{name:34s} bit-exact {sum(ok)}/{len(rows)}  within 4 ulp {within}")


main()
