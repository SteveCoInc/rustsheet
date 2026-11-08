"""T-4419 search on probes/t4419-band-flip (169 cells): 13 decimal millisecond ties t/86400000 + n*2^-71 days, n from
-20 to -8, under "s.000". Excel's switch point n is monotone for every tie. Run from the repository root:
python3 docs/research/t4419/chains.py.

For each ordered grouping of the factors 24, 60, 60 and 1000, this computes the product as a chain of doubles, rounds the
milliseconds to 15 significant digits, then rounds half up to a whole millisecond. The best two chains get 166 of 169,
and they miss disjoint ties: (x*86400)*1000 misses 255.5, 316.5 and 320.5; ((x*24)*60)*60*1000 misses 258.5, 266.5 and
291.5. `windows()` prints the error each tie needs in the computed seconds, in units of 5e-16 s, the half unit of
15-digit seconds, for one threshold at -1 to hold. They run from -0.17 (320.5) to +0.15 (255.5), about one s-ulp
either side, so no single rounding error explains them.
"""

import itertools
import json
import re
from decimal import ROUND_HALF_UP, Decimal, getcontext
from fractions import Fraction

getcontext().prec = 70
CASE = "eval/suites/probes/cases/t4419-band-flip"
cells = json.load(open(CASE + ".case.json"))["sheets"][0]["cells"]
oracle = json.load(open(CASE + ".oracle.json"))["sheets"]["Sheet1"]
PAT = re.compile(r'=TEXT\(([\d.]+)/86400000\+\((-?\d+)\)\*2\^\(-71\),"s.000"\)')
rows = []
for cell, formula in cells.items():
    t, n = PAT.match(formula).groups()
    rows.append((float(t) / 86400000 + int(n) * 2.0**-71, Decimal(t), int(n), oracle[cell]["v"]))


def r15(v):
    d = Decimal(v)
    return d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_HALF_UP)


def groupings(fs):
    if not fs:
        yield ()
        return
    for i in range(1, len(fs) + 1):
        head = 1.0
        for f in fs[:i]:
            head *= f
        for rest in groupings(fs[i:]):
            yield (head, *rest)


def score(seq):
    ok, miss = 0, set()
    for x, t, _, want in rows:
        m = x
        for f in seq:
            m *= f
        ms = int(r15(m).quantize(Decimal(1), rounding=ROUND_HALF_UP))
        if f"{ms / 1000:.3f}" == want:
            ok += 1
        else:
            miss.add(float(t))
    return ok, sorted(miss)


def windows():
    by_tie = {}
    for x, t, n, want in rows:
        by_tie.setdefault(t, []).append((n, want == f"{(t + Decimal('0.5')) / 1000:.3f}", x))
    for t, r in sorted(by_tie.items()):
        up = min(n for n, u, _ in r if u)
        pos = {n: float((Fraction(x) * 86400 - Fraction(t) / 1000) / Fraction(5, 10**16)) for n, _, x in r}
        print(f"tie {t}: switch at n={up}; needed error in [{-1 - pos[up]:+.3f}, {-1 - pos[up - 1]:+.3f})")


if __name__ == "__main__":
    seqs = {g for p in itertools.permutations([24.0, 60.0, 60.0, 1000.0]) for g in groupings(list(p))}
    for ok, seq, miss in sorted(((*score(s), s) for s in seqs), reverse=True)[:4]:
        print(ok, seq, miss)
    windows()
