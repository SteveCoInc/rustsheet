"""T-4389 scores: python3 score.py [META:CASE ...]. Default: the train cells (the `.000` cells of the t4323/t4332/
t4370 cases the constant was fitted on) and the hold-out probes/t4389-holdout."""
import math
import sys
from fractions import Fraction

from common import base_digits, days_digits, fields, fmt, load, seconds_digits


def correctly_rounded(x, s):
    units = math.floor(Fraction(x) * 86400 * s + Fraction(1, 2))
    return units % s, (units // s) % 86400, units // s


T = "../t4370/"
CANDS = (("C in days (frozen)", fields(days_digits)), ("C in seconds", fields(seconds_digits)),
         ("T-4370", fields(base_digits)), ("correctly rounded", correctly_rounded))


def report(label, rows):
    print(f"{label}: {len(rows)} cells")
    for name, fn in CANDS:
        bad = [r[0] for r in rows if fmt(r[1], r[2], fn(r[3], r[2])) != r[4]]
        print(f"  {name:22} {len(rows) - len(bad)}/{len(rows)}  misses {bad[:6]}{' ...' if len(bad) > 6 else ''}")


if __name__ == "__main__":
    if sys.argv[1:]:
        report("given", load([tuple(a.split(":")) for a in sys.argv[1:]]))
    else:
        from model import CASES
        train = load([(T + "../t4332/" + m, c) for m, c in CASES] +
                     [(T + "../t4332/meta6.json", "t4332-fraction-fresh"), (T + "meta7.json", "t4370-ulp-map"),
                      (T + "meta8.json", "t4370-ulp-map-2"), (T + "meta9.json", "t4370-fraction-fresh")])
        report("train, .000 cells", [r for r in train if r[2] == 1000])
        report("train, all cells", train)
        hold = load([("../t4389/meta10.json", "t4389-holdout")])
        report("hold-out, all cells", hold)
        report("hold-out, .000 cells", [r for r in hold if r[2] == 1000])
