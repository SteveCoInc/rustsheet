"""R4.1 hold-out of T-4323: exact rates of the half-step register (fu.py's model) and of the correctly
rounded value (the double x rounded exactly to the shown digit), train cases vs the hold-out case."""
import math
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
for name, pairs in (("train", TRAIN), ("hold-out", HOLD)):
    rows = load(pairs)
    for label, m in (("half-step", fu), ("correctly rounded", correctly_rounded)):
        ok = sum(fmt(f, s, m((d + k / 86400) if d else k / 86400, s)) == e for (_, _, f, s, d, k, e) in rows)
        print(f"{name:9} {label:18} {ok}/{len(rows)} = {100 * ok / len(rows):.1f}%")
