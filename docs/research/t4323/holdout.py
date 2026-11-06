"""R4.1 hold-out of T-4323: exact rates of the half-step register (fu.py's model) and of the correctly
rounded value (the double x rounded exactly to the shown digit), train cases vs the hold-out case."""
import hashlib
import math
import sys
from fractions import Fraction

from eps import load
from frozen_fmt import fmt
from fu import fu


def correctly_rounded(x, s):
    units = math.floor(Fraction(x) * 86400 * s + Fraction(1, 2))
    return units % s, (units // s) % 86400, units // s


TRAIN = [("meta1.json", "t4323-fraction-ties"), ("meta2.json", "t4323-fraction-sweep"),
         ("meta3.json", "t4323-fraction-sweep-2")]
HOLD = [("meta5.json", "t4323-fraction-holdout")]
SPLITS = {"train": load(TRAIN), "hold-out": load(HOLD)}
if "--pooled" in sys.argv:
    # decision 0188 R4.1: ~30% of every cell of the five cases by hash, fixed before scoring.
    rows = load(TRAIN + [("meta4.json", "t4323-fraction-sweep-3")] + HOLD)
    def held(row):
        return int(hashlib.sha256(f"{row[0]}!{row[1]}".encode()).hexdigest(), 16) % 100 < 30
    SPLITS = {"train": [r for r in rows if not held(r)], "hold-out": [r for r in rows if held(r)]}
for name, rows in SPLITS.items():
    for label, m in (("half-step", fu), ("correctly rounded", correctly_rounded)):
        ok = sum(fmt(f, s, m((d + k / 86400) if d else k / 86400, s)) == e for (_, _, f, s, d, k, e) in rows)
        print(f"{name:9} {label:18} {ok}/{len(rows)} = {100 * ok / len(rows):.1f}%")
