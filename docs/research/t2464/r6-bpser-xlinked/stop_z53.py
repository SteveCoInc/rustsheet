"""With z stored, does an earlier stop of w3's series explain the all-(+1) misses? The tail terms are positive once
n > b, so leaving some out lowers s. Rules: stop before adding a term w with |w| <= tol, tol in
{eps/a (TOMS bpser), eps*s} for the structural eps candidates DBL_EPSILON, 1e-15 (TOMS bratio's floor), 1e-14;
"none" = w3's "until s stops changing". Bit-exact per (b0, center).
usage: stop_z53.py"""
from collections import defaultdict

import common as cm

sx = cm.sx
EPSS = {"2^-52": 2.0**-52, "1e-15": 1e-15, "1e-14": 1e-14, "2^-53": 2.0**-53}


def series_stop(a, b, x, rule, eps):
    s = sx("/", 1.0, a)
    c, n = 1.0, 0.0
    for _ in range(200000):
        n += 1.0
        c = sx("*", sx("*", c, sx("+", sx("-", 0.5, sx("/", b, n)), 0.5)), x)
        w = sx("/", c, sx("+", a, n))
        tol = eps / a if rule == "eps/a" else eps * s
        if rule != "none" and abs(w) <= tol and n > b:
            break
        s2 = sx("+", s, w)
        if s2 == s:
            break
        s = s2
    return s


def main() -> None:
    rules = [("none", 0.0)] + [(r, e) for r in ("eps/a", "eps*s") for e in EPSS.values()]
    names = ["none"] + [f"{r} {k}" for r in ("eps/a", "eps*s") for k in EPSS]
    score = defaultdict(lambda: defaultdict(int))
    for blk, x, a, b, ex in cm.r5_rows():
        key = ":".join(blk.split(":")[:2])
        e = cm.EXP(cm.pieces(a, b, x)["z53"])
        for nm, (r, ep) in zip(names, rules):
            score[nm][key] += sx("*", e, series_stop(a, b, x, r, ep)) == ex
    keys = sorted({k for d in score.values() for k in d})
    print(f"{'stop':14}" + "".join(f"{k:>10}" for k in keys) + f"{'total':>8}")
    for nm in names:
        d = score[nm]
        print(f"{nm:14}" + "".join(f"{d[k]:>10}" for k in keys) + f"{sum(d.values()):>8}")


if __name__ == "__main__":
    main()
