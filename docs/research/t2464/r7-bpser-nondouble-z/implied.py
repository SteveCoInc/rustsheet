"""Per non-double-z cell: Excel's E set (doubles E with fl(E s) = Excel's I), placed against E64 = EXP(extended z),
Elo = EXP(z rounded down) and Ehi = EXP(z rounded up), in units of ulp(E64). Prints per shape the counts of the
implied position of Excel's E: 'lo' / 'hi' / 'x' (= E64) and offsets of the E set from E64.
usage: implied.py"""
import math
from collections import Counter, defaultdict

import c7

cm = c7.cm


def main() -> None:
    tab = defaultdict(Counter)
    for c in c7.cells():
        if not c["nd"]:
            continue
        e64 = cm.EXPX(cm.fl64(c["z"]))
        u = math.ulp(e64)
        ks = [round((E - e64) / u) for E in c7.e_ok(c)]
        lo = round((cm.EXP(c["zlo"]) - e64) / u)
        hi = round((cm.EXP(math.nextafter(c["zlo"], math.inf)) - e64) / u)
        tab[c7.shape(c)][f"lo{lo:+d} hi{hi:+d} q{c['q']:.2f} E-set {ks}"] += 1
    for k in sorted(tab):
        print(k)
        for kk, n in sorted(tab[k].items()):
            print(f"   {n:4d}  {kk}")


if __name__ == "__main__":
    main()
