"""How Excel combines the prefactor and the series, scored per shape on t3769-r6-bpser-beta with measured GAMMALN
(L = GL53's L; at a0 = 0.5 also with g = GAMMALN + 1 ulp, gscan.out). ia = fl(1/a), sp = the series without 1/a
(w3's terms, summed forward from 0, stored), x^a = EXP(alx), B^-1 = EXP(-L).
usage: product.py"""
from collections import defaultdict

import common as cm

sx, fl64, mpf, EXP, EXPX = cm.sx, cm.fl64, cm.mpf, cm.EXP, cm.EXPX


def sp_terms(a, b, x):
    c, n, sp, ia = 1.0, 0.0, 0.0, sx("/", 1.0, a)
    for _ in range(200000):
        n += 1.0
        c = sx("*", sx("*", c, sx("+", sx("-", 0.5, sx("/", b, n)), 0.5)), x)
        t = sx("/", c, sx("+", a, n))
        if sx("+", ia, sx("+", sp, t)) == sx("+", ia, sp) and abs(t) < 1e-25:
            break
        sp = sx("+", sp, t)
    return ia, sp


def forms(a, b, x, g_shift):
    g = cm.step(cm.GAMMALN(a), g_shift)
    L = sx("+", g, sx("-", cm.GAMMALN(b), cm.GAMMALN(sx("+", a, b))))
    alx = sx("*", a, cm.LN(x))
    E = EXPX(fl64(mpf(alx) - mpf(L)))
    s = cm.MODEL.series_w3(a, b, x)
    ia, sp = sp_terms(a, b, x)
    xa, binv = EXP(alx), EXP(-L)
    return {
        "E*s": sx("*", E, s),
        "E*ia + E*sp": sx("+", sx("*", E, ia), sx("*", E, sp)),
        "E/a + E*sp": sx("+", sx("/", E, a), sx("*", E, sp)),
        "(x^a*s)*B^-1": sx("*", sx("*", xa, s), binv),
        "x^a*(s*B^-1)": sx("*", xa, sx("*", s, binv)),
        "(x^a*B^-1)*s": sx("*", sx("*", xa, binv), s),
        "(x^a/B)*s": sx("*", sx("/", xa, EXP(L)), s),
        "(x^a*s)/B": sx("/", sx("*", xa, s), EXP(L)),
    }


def main() -> None:
    score = defaultdict(lambda: defaultdict(int))
    for part, b, a, x, e in cm.beta_rows():
        key = f"{part[0]}{b}:{a}"
        for sh in ((0, 1) if a == 0.5 else (0,)):
            for nm, v in forms(a, b, x, sh).items():
                score[nm + (" g+1" if sh else "")][key] += v == e
    keys = sorted({k for d in score.values() for k in d}, key=lambda k: (k.split(":")[1], k))
    print(f"{'form':22}" + "".join(f"{k:>9}" for k in keys))
    for nm, d in score.items():
        print(f"{nm:22}" + "".join(f"{d.get(k, ''):>9}" for k in keys))


if __name__ == "__main__":
    main()
