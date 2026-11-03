"""Score GL53 variants on r5's probe per (b0, center): the series order/form (r2's lib.series), alx's precision
(lib.V.alx), z kept extended or stored (lib.V.z) and E kept extended into the product (lib.V.eext).
Bit-exact counts per block; tune = b0 2.5, hold-out = b0 6.25 (r5's split).
usage: variants_r5.py"""
from collections import defaultdict

import common as cm

SERIES = ("fwd1a:toms", "fwd0:toms", "cephes:toms", "pair:toms", "fwd1a:mul", "fwd1a:1-b/n", "fwd1a:(n-b)/n",
          "fwd1a:(n-b)x/n")
VARIANTS = {"GL53": cm.GL53}
for alx in ("64", "ln64"):
    VARIANTS[f"alx{alx}"] = cm.lib.V(lg="GL53", alx=alx)
VARIANTS["z53"] = cm.lib.V(lg="GL53", z="z53")
VARIANTS["eext"] = cm.lib.V(lg="GL53", eext=True)
VARIANTS["eext+z53"] = cm.lib.V(lg="GL53", eext=True, z="z53")


def main() -> None:
    rows = cm.r5_rows()
    score = defaultdict(lambda: defaultdict(int))
    for blk, x, a, b, e in rows:
        key = ":".join(blk.split(":")[:2])
        for nm, v in VARIANTS.items():
            score[nm][key] += cm.lib.kernel(a, b, x, 1.0 - x, v)[0] == e
        p = cm.pieces(a, b, x)
        for sn in SERIES:
            score["ser " + sn][key] += cm.sx("*", p["e"], cm.lib.series(sn, a, b, x)) == e
    keys = sorted({k for d in score.values() for k in d})
    print(f"{'variant':22}" + "".join(f"{k:>10}" for k in keys) + f"{'total':>8}")
    for nm, d in score.items():
        print(f"{nm:22}" + "".join(f"{d[k]:>10}" for k in keys) + f"{sum(d.values()):>8}")


if __name__ == "__main__":
    main()
