"""With z stored (E = EXP(fl53 z)), score series/product forms on r5's probe per (b0, center):
  st     w3's series, every op stored (the GL53 model)
  ext    the same recurrence with every op kept extended (fl64), s stored at the end, I = fl(E s)
  extp   ext, but s stays extended into the product: I = fl53(fl64(E s64))
  cr     s correctly rounded from the exact series (accuracy control)
  toms   TOMS's form I = fl(fl(E/a) fl(fl(a s') + 1)), s' = sum without 1/a, stored
  cext   c kept extended, the sum stored
  sext   c stored, the sum kept extended (s stored at the end)
usage: series_z53.py"""
from collections import defaultdict

import mpmath

import acc_r5
import common as cm

fl64, fl53, mpf, sx = cm.fl64, cm.fl53, cm.mpf, cm.sx


def series_mixed(a, b, x, cx: bool, sxt: bool):
    """w3's recurrence; cx: c and its factor kept extended; sxt: the running sum kept extended. Returns s64."""
    def op(ext, v):
        return fl64(v) if ext else mpf(fl53(fl64(v)))
    s = op(sxt, mpf(1) / mpf(a))
    c, n = mpf(1), 0
    for _ in range(200000):
        n += 1
        f = op(cx, op(cx, mpf(0.5) - op(cx, mpf(b) / n)) + mpf(0.5))
        c = op(cx, op(cx, c * f) * mpf(x))
        s2 = op(sxt, s + op(cx, c / op(cx, mpf(a) + n)))
        if s2 == s:
            break
        s = s2
    return s


def forms(a, b, x, e):
    em = mpf(e)
    out = {}
    st = cm.MODEL.series_w3(a, b, x)
    out["st"] = sx("*", e, st)
    s64 = series_mixed(a, b, x, True, True)
    out["ext"] = sx("*", e, fl53(s64))
    out["extp"] = fl53(fl64(em * s64))
    out["cext"] = sx("*", e, fl53(series_mixed(a, b, x, True, False)))
    out["sext"] = sx("*", e, fl53(series_mixed(a, b, x, False, True)))
    out["cr"] = sx("*", e, float(acc_r5.s_exact(a, b, x)))
    sp = cm.lib.Model.series_toms(a, b, x, cm.lib.EPS)
    out["toms"] = sx("*", sx("/", e, a), sx("+", sx("*", a, sp), 1.0))
    return out


def main() -> None:
    mpmath.mp.prec = 200
    score = defaultdict(lambda: defaultdict(int))
    for blk, x, a, b, ex in cm.r5_rows():
        key = ":".join(blk.split(":")[:2])
        p = cm.pieces(a, b, x)
        for nm, v in forms(a, b, x, cm.EXP(p["z53"])).items():
            score[nm][key] += v == ex
    keys = sorted({k for d in score.values() for k in d})
    print(f"{'form':8}" + "".join(f"{k:>10}" for k in keys) + f"{'total':>8}")
    for nm, d in score.items():
        print(f"{nm:8}" + "".join(f"{d[k]:>10}" for k in keys) + f"{sum(d.values()):>8}")


if __name__ == "__main__":
    main()
