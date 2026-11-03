"""The cells the split model (I = fl(fl(EXP(zh) s) fl(1 + zl)), zh = z to nearest) misses on r6 and r5, with the
error in ulps of Excel's I; and its exact / within-4-ulp / within-15-digit counts per probe part.
usage: misses.py"""
import math
from collections import Counter

import c7
import forms

KEY = forms.KEY


def main() -> None:
    for probe in ("r6", "r5"):
        n = Counter()
        for c in c7.cells(probe):
            got, want = forms.forms(c)[KEY], c["I"]
            d = (got - want) / math.ulp(want)
            n[(c["part"], "cells")] += 1
            n[(c["part"], "exact")] += got == want
            n[(c["part"], "4ulp")] += c7.cm.lib.sl.ulpd(got, want) <= 4
            n[(c["part"], "15dig")] += c7.cm.lib.sl.w15(got, want)
            if got != want:
                print(f"miss {c7.shape(c)} x={c['x']!r} nd={c['nd']} q={c['q']} excel={want!r} model={got!r} "
                      f"({d:+.0f} ulp)")
        for part in sorted({k[0] for k in n}):
            print(f"{probe} {part}: cells {n[(part, 'cells')]}, exact {n[(part, 'exact')]}, within 4 ulp "
                  f"{n[(part, '4ulp')]}, within 15 digits {n[(part, '15dig')]}")


if __name__ == "__main__":
    main()
