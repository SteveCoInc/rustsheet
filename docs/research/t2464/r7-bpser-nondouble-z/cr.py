"""Hold-out record (decision 0188 R4.1): the correctly rounded value (CR, mpmath betainc at 50 digits, rounded to a
double) beside the split model, exact / within 4 ulp / within 15 digits (r2-bratio score_lib's ulpd and w15), on r6's
tune and hold-out parts and on r5.
usage: cr.py"""
from collections import Counter

import c7
import forms
import mpmath

sl = c7.cm.lib.sl


def main() -> None:
    t = Counter()
    for probe in ("r6", "r5"):
        for c in c7.cells(probe):
            with mpmath.workdps(50):
                cr = float(mpmath.betainc(c["a"], c["b"], 0, c["x"], regularized=True))
            t[(c["part"], "n")] += 1
            for nm, v in (("CR", cr), ("split", forms.forms(c)[forms.KEY])):
                t[(c["part"], nm, "exact")] += v == c["I"]
                t[(c["part"], nm, "4ulp")] += sl.ulpd(v, c["I"]) <= 4
                t[(c["part"], nm, "15d")] += sl.w15(v, c["I"])
    for part in ("tune", "hold", "r5"):
        row = [f"{t[(part, nm, k)]:,}" for nm in ("CR", "split") for k in ("exact", "4ulp", "15d")]
        print(f"{part:5s} cells {t[(part, 'n')]:,}  CR {' / '.join(row[:3])}  split {' / '.join(row[3:])}")


if __name__ == "__main__":
    main()
