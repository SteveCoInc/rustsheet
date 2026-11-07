"""Score GL53 with z kept extended (z64, r5's model) against z stored (z53, found on r5's data by variants_r5.py) on
the fresh probe t3769-r6-bpser-beta: exact / within 4 ulp / within 15 digits per shape, tune (b0 2.5, 6.25) and
hold-out (b0 4.75), split fixed in make_probe.py before the fill. Misses: signed offsets (model - Excel, ulps).
usage: score_z.py"""
from collections import Counter, defaultdict

import common as cm

FORMS = {"z64": cm.GL53, "z53": cm.lib.V(lg="GL53", z="z53")}


def main() -> None:
    rows = cm.beta_rows()
    res = defaultdict(list)
    offs = defaultdict(Counter)
    for part, b, a, x, e in rows:
        for nm, v in FORMS.items():
            m = cm.lib.kernel(a, b, x, 1.0 - x, v)[0]
            res[(nm, f"{part} b0={b} a0={a}")].append((m, e))
            res[(nm, part)].append((m, e))
            if m != e:
                offs[(nm, part)][round((m - e) / cm.math.ulp(e))] += 1
    assert not cm.lib.bx._GL_MISSING
    keys = sorted({k for _, k in res}, key=lambda k: (len(k), k))
    print(f"{'set':26}" + "".join(f"{nm:>18}" for nm in FORMS))
    for k in keys:
        print(f"{k:26}" + "".join(f"{cm.lib.fmt(cm.lib.tally(res[(nm, k)])):>18}" for nm in FORMS))
    for k, c in sorted(offs.items()):
        print("offsets", k, dict(sorted(c.items())))


if __name__ == "__main__":
    main()
