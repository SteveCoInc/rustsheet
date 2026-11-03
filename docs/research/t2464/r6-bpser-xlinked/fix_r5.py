"""On r5's probe, which stored piece moved by k ulps (|k| <= 2) fixes each GL53 miss: s, E, or z53 (E = EXP(fl53 z))?
usage: fix_r5.py"""
from collections import Counter, defaultdict

import common as cm


def main() -> None:
    tab = defaultdict(Counter)
    for blk, x, a, b, e in cm.r5_rows():
        p = cm.pieces(a, b, x)
        assert p["i"] == cm.lib.kernel(a, b, x, 1.0 - x, cm.GL53)[0]
        if p["i"] == e:
            tab[blk]["exact"] += 1
            continue
        fx = []
        for k in (-2, -1, 1, 2):
            if cm.sx("*", p["e"], cm.step(p["s"], k)) == e:
                fx.append(f"s{k:+d}")
            if cm.sx("*", cm.step(p["e"], k), p["s"]) == e:
                fx.append(f"E{k:+d}")
        tab[blk][" ".join(fx) or "none"] += 1
    for blk in sorted(tab):
        print(blk, dict(tab[blk].most_common()))


if __name__ == "__main__":
    main()
