"""T-3774 fit attempt 2, step 1: is the rest of attempt 1's fit (rms 0.77 |d| 2^-53) constant along each run (k, n
fixed, p by consecutive doubles)? A per-run constant would come from a piece that does not depend on p (lf or the
stirlerr terms), not from bd0. Fits attempt 1's four terms, then reports the rest in absolute units of 2^-53 (rest
times |d|): its total rms, and its rms after removing each run's mean.
usage: tools/.venv/bin/python -I docs/research/t3774/runconst.py SHOW_FILE  (from the repo root)"""
from __future__ import annotations

import importlib.util
import json
import math
import re
import sys
from collections import defaultdict
from pathlib import Path

spec = importlib.util.spec_from_file_location("pr", Path("docs/research/t3447/pmf_regress.py"))
pr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pr)
mpf = pr.mpf


def rows(show: str):
    cells = json.load(open(pr.CASE + ".case.json"))["sheets"][0]["cells"]
    oracle = json.load(open(pr.CASE + ".oracle.json"))["sheets"]["Calc"]
    engine = {m[1]: float(m[2]) for m in re.finditer(r"!(B\d+).*?\n.*?\n.*?\n\s+actual\s*:\s*(\S+)", show)}
    for addr, formula in cells.items():
        m = re.match(r"=BINOM\.DIST\((\d+),(\d+),(.*),FALSE\)$", formula)
        if not (addr.startswith("B") and m and int(m[2]) >= 10000 and int(m[1]) > 100):
            continue
        excel = oracle.get(addr, {})
        if excel.get("t") != "n" or excel["v"] == 0:
            continue
        k, n, p = float(m[1]), float(m[2]), pr.literal(m[3])
        q = float(1 - mpf(p))
        nk = n - k
        exact_p, exact_q = mpf(n) * mpf(p), mpf(n) * (1 - mpf(p))
        m_p, m_q = pr.rnd(exact_p, 53), pr.rnd(mpf(n) * mpf(q), 53)
        d = abs(float(k - m_p))
        if d < 20:
            continue
        lf = pr.log(2 * pr.pi) + pr.log(mpf(k)) + pr.log(mpf(nk)) - pr.log(mpf(n))
        ref = pr.exp(pr.stirlerr(n) - pr.stirlerr(k) - pr.stirlerr(nk) - pr.bd0(k, exact_p) - pr.bd0(nk, exact_q) - lf / 2)
        scale = 1 / (d * 2.0**-53)
        x = [-float(t) * scale for t in pr.side(k, exact_p, m_p) + pr.side(nk, exact_q, m_q)]
        yield (k, n), d, x, float((excel["v"] - ref) / ref) * scale, float((engine[addr] - ref) / ref) * scale, (k, nk, m_p, m_q, scale)


def main() -> None:
    data = list(rows(open(sys.argv[1]).read()))
    xs = [r[2] for r in data]
    w = len(pr.NAMES)
    cols = [0, w, 2, w + 2]
    for label, yi in (("excel", 3), ("engine", 4)):
        ys = [r[yi] for r in data]
        coef, _ = pr.lstsq(xs, ys, cols)
        rest = defaultdict(list)
        for r, x, y in zip(data, xs, ys):
            rest[r[0]].append((y - sum(c * x[ci] for c, ci in zip(coef, cols))) * r[1])
        allv = [v for vs in rest.values() for v in vs]
        within = [v - sum(vs) / len(vs) for vs in rest.values() for v in vs]
        rms = lambda a: math.sqrt(sum(t * t for t in a) / len(a))  # noqa: E731
        print(f"{label}: {len(rest)} runs, {len(allv)} cells; rest in 2^-53 units: rms {rms(allv):.1f}, "
              f"after each run's mean {rms(within):.1f}")
        for key, vs in sorted(rest.items()):
            mean = sum(vs) / len(vs)
            print(f"    k={key[0]:.0f} n={key[1]:.0f} cells={len(vs)} mean {mean:+.1f} rms within {rms([v - mean for v in vs]):.1f}")


if __name__ == "__main__":
    main()
