"""The same split as repr_r5.py on t3769-r6-bpser-beta (GL53, L from Excel's GAMMALN): per shape, GL53's misses on cells
whose z = alx - L is a double and on the rest; and at a0 = 0.5 the same with g = GAMMALN + 1 ulp (gscan.out's best
double g), to test whether that g can be Excel's (a right L leaves no miss on double-z cells).
usage: repr_r6.py"""
from collections import Counter, defaultdict

import common as cm

sx, mpf = cm.sx, cm.mpf


def main() -> None:
    tab = defaultdict(Counter)
    for part, b, a, x, e in cm.beta_rows():
        s = cm.MODEL.series_w3(a, b, x)
        alx = sx("*", a, cm.LN(x))
        d = sx("-", cm.GAMMALN(b), cm.GAMMALN(sx("+", a, b)))
        for gs in ((0, 1) if a == 0.5 else (0,)):
            z = mpf(alx) - mpf(sx("+", cm.step(cm.GAMMALN(a), gs), d))
            rep = "double z" if mpf(float(z)) == z else "non-double z"
            ok = sx("*", cm.EXPX(cm.fl64(z)), s) == e
            tab[(f"{part} b0={b} a0={a}" + (" g+1" if gs else ""), rep)][ok] += 1
            if not gs:
                tab[("all GL53", rep)][ok] += 1
    for k in sorted(tab, key=lambda k: (k[0].startswith("all"), k)):
        c = tab[k]
        print(f"{k[0]:26} {k[1]:13} miss {c[False]:>4} / {c[False] + c[True]:>4}")


if __name__ == "__main__":
    main()
