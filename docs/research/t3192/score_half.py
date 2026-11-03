"""T-3192: score a = b = 1/2 closed forms on Excel's cells.

Sweep 1: probes-numeric/t3192-beta-half-half (column B BETA.DIST; C BETADIST equals it on every row).
Sweep 2: probes-numeric/t3192-beta-half-half-2 (column B), filled after the 0.9 switch was frozen on sweep 1.
Sweep 3: probes-numeric/t3192-beta-half-half-3 (column B), filled after review round 2 froze the direct form.
`Er` was the choice frozen on the 49 cells on file before T-3192 (reflection at x > 1/2); sweep 1 rejected it and
fixed the switch at 0.9 (`switch 0.9`), which sweep 2 tests; review round 2 dropped the switch and ported the direct
form (`incomplete_beta::half_half_cdf`).
Usage (repo root): tools/.venv/bin/python docs/research/t3192/score_half.py
"""
import hashlib
import json
import math
from pathlib import Path

from mpmath import asin, atan, betainc, mp, mpf

mp.dps = 40
CASE = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half")
CASE2 = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half-2")
CASE3 = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half-3")
sq = math.sqrt


def refl(f):
    return lambda x: f(x) if x <= 0.5 else (0.5 - f(1 - x)) + 0.5


FORMS = {
    "switch 0.9 (not ported)": lambda x: (
        2 * math.atan(sq(x) / sq(1 - x)) / math.pi
        if x <= 0.9
        else (0.5 - 2 * math.atan(sq(1 - x) / sq(x)) / math.pi) + 0.5
    ),
    "Er (frozen)": refl(lambda x: 2 * math.atan(sq(x) / sq(1 - x)) / math.pi),
    "E direct (ported)": lambda x: 2 * math.atan(sq(x) / sq(1 - x)) / math.pi,
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
    print("sweep 3 (R4.2 fresh sweep of the direct form)")
    score(load(CASE3))
    holdout()


def is_holdout(case, key):
    """The fit lab's split (tools/src/rustsheet_tools/fitlab/holdout.py, decision 0188 R4.1), copied so this
    script runs without the tools package; the case id is `suite/name`, the key `Calc!B12`."""
    digest = hashlib.sha256(f"rustsheet-fitlab-holdout/v1\0{case}\0{key}".encode()).digest()
    return int.from_bytes(digest[:8], "big") % 1000 < 300


def holdout():
    """Train and test (the fit lab's ~30% split) over both sweeps' B and C cells; margin 5 points (DEFAULT_MARGIN),
    stated before scoring."""
    cells = {"train": [], "test": []}
    for path in (CASE, CASE2):
        case_id = f"probes-numeric/{path.name}"
        for i, (x, b, c, _) in enumerate(load(path), start=1):
            for col, v in (("B", b), ("C", c)):
                if isinstance(v, float):
                    side = "test" if is_holdout(case_id, f"Calc!{col}{i}") else "train"
                    cells[side].append((x, v))
    for name in ("E direct (ported)", "switch 0.9 (not ported)", "correctly rounded"):
        f = FORMS[name]
        line = []
        for side, rows in cells.items():
            exact = sum(f(x) == v for x, v in rows)
            ulp4 = sum(abs(f(x) - v) <= 4 * math.ulp(v) for x, v in rows)
            d15 = sum(abs(f(x) - v) <= 5e-16 * abs(v) for x, v in rows)
            line.append(f"{side} {exact}/{ulp4}/{d15} of {len(rows)} ({exact / len(rows):.1%})")
        print(f"fit-lab split {name:22s} " + "; ".join(line))


def score(rows):
    for name, f in FORMS.items():
        ok = [f(x) == b for x, b, _, _ in rows]
        within = sum(abs(f(x) - b) <= 4 * math.ulp(b) for x, b, _, _ in rows)
        print(f"{name:34s} bit-exact {sum(ok)}/{len(rows)}  within 4 ulp {within}")


main()
