"""The model families T-4419 searched, scored on probes/t4419-band-flip (169 cells) and probes/t4419-band-dense
(1,148). Run from the repository root: python3 docs/research/t4419/family_search.py (a few minutes). Each family's best
member is printed with its score on both probes. A member maps the serial x to a millisecond count, and the shown digits
are that count mod 1000.

- ops: x through 24, 60, 60 and 1000 in any order, each step `*f` or `/(1/f)` (division by the inexact reciprocal),
  then 15-digit rounding and half up (chains.py's last step).
- grouped: the same with the factors grouped (86400, 3600, 1440, 86400000, ...), first step `*f` or `/(1/f)`.
- directed: the grouped chains with each step's product rounded to nearest, down or up, at 53 or 64 bits.
- half-cut: a half step in days (0.5/86400000 and its other spellings) added first, the product chain, then a cut:
  floor, floor of the 15-digit value, or floor of the 15-digit seconds.
"""

import itertools
import json
import math
import re
from decimal import ROUND_HALF_UP, Decimal, getcontext
from fractions import Fraction

getcontext().prec = 60
PAT = re.compile(r'=TEXT\(([\d.]+)/86400000\+\((-?\d+)\)\*2\^\(-71\),"s.000"\)')


def load(name):
    base = "eval/suites/probes/cases/" + name
    cells = json.load(open(base + ".case.json"))["sheets"][0]["cells"]
    oracle = json.load(open(base + ".oracle.json"))["sheets"]["Sheet1"]
    out = []
    for cell, formula in cells.items():
        t, n = PAT.match(formula).groups()
        out.append((float(t) / 86400000 + int(n) * 2.0**-71, int(oracle[cell]["v"].split(".")[1])))
    return out


PROBES = {"band-flip": load("t4419-band-flip"), "band-dense": load("t4419-band-dense")}


def r15(v):
    d = Decimal(v)
    return d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_HALF_UP)


def half_up(m):
    return int(r15(m).quantize(Decimal(1), rounding=ROUND_HALF_UP)) % 1000


def to_bits(q, mode, bits):
    """The Fraction q rounded to a binary float of `bits` mantissa bits: to nearest (even), down or up."""
    e = math.floor(math.log2(abs(q)))
    while Fraction(2) ** e > abs(q):
        e -= 1
    while Fraction(2) ** (e + 1) <= abs(q):
        e += 1
    v = q / Fraction(2) ** (e - bits + 1)
    r = {"near": round(v), "down": math.floor(v), "up": math.ceil(v)}[mode]
    return r * Fraction(2) ** (e - bits + 1)


def score(fn):
    return {name: sum(fn(x) == want for x, want in rows) for name, rows in PROBES.items()}


def members():
    for perm in sorted(set(itertools.permutations([24.0, 60.0, 60.0, 1000.0]))):
        for ops in itertools.product("*/", repeat=4):
            label = "".join(o + str(int(f)) for f, o in zip(perm, ops))

            def fn(x, perm=perm, ops=ops):
                m = x
                for f, o in zip(perm, ops):
                    m = m * f if o == "*" else m / (1 / f)
                return half_up(m)

            yield "ops", label, fn
    grouped = [(86400.0, 1000.0), (24.0, 3600.0, 1000.0), (1440.0, 60.0, 1000.0), (24.0, 60.0, 60000.0),
               (86400000.0,), (3600.0, 24000.0), (24000.0, 3600.0)]
    for seq in grouped:
        for first in "*/":
            label = first + "*".join(str(int(f)) for f in seq)

            def fn(x, seq=seq, first=first):
                m = x
                for i, f in enumerate(seq):
                    m = m / (1 / f) if i == 0 and first == "/" else m * f
                return half_up(m)

            yield "grouped", label, fn
        for modes in itertools.product(["near", "down", "up"], repeat=len(seq)):
            for bits in (53, 64):

                def fn(x, seq=seq, modes=modes, bits=bits):
                    m = Fraction(x)
                    for f, md in zip(seq, modes):
                        m = to_bits(m * Fraction(f), md, bits)
                    return half_up(Decimal(m.numerator) / Decimal(m.denominator))

                yield "directed", f"{seq} {modes} {bits}-bit", fn
    halves = {"0.5/86400000": 0.5 / 86400000, "0.0005/86400": 0.0005 / 86400, "0.5/(86400*1000)": 0.5 / (86400 * 1000)}
    for (hn, h), seq in itertools.product(halves.items(), sorted(set(itertools.permutations([24.0, 60.0, 60.0, 1000.0])))):
        for cut in ("floor", "r15 floor", "r15 seconds floor"):

            def fn(x, h=h, seq=seq, cut=cut):
                m = x + h
                for f in seq:
                    m *= f
                if cut == "floor":
                    return math.floor(m) % 1000
                if cut == "r15 floor":
                    return int(math.floor(r15(m))) % 1000
                return int(math.floor(r15(m / 1000) * 1000)) % 1000

            yield "half-cut", f"+{hn} {'*'.join(str(int(f)) for f in seq)} {cut}", fn


if __name__ == "__main__":
    best = {}
    for family, label, fn in members():
        s = score(fn)
        if family not in best or s["band-dense"] > best[family][0]["band-dense"]:
            best[family] = (s, label)
        if label == "/24*60*60*1000":
            print("x/(1/24)*60*60*1000:", s)
    for family, (s, label) in best.items():
        print(f"{family:9} best on band-dense: {label}  {s}")
