"""T-3771: the double-z test pooled over cells (lforms.py's per-a0 'consistent' leaves an a0 unjudged when a form puts
none of its z on the double grid). Per form (lforms.forms) and center band, tune/hold-out: double-z cells and how many
of them miss. A right L misses none (T-3769); a wrong L misses some of them.
usage: pooled.py"""
from __future__ import annotations

from collections import Counter, defaultdict

import lforms

pin = lforms.pin


def band(a0: float) -> str:
    return "0.15-0.4" if a0 < 0.45 else "0.5-0.7" if a0 < 0.75 else "0.85"


def main() -> None:
    aa = lforms.aux_a()
    tab = defaultdict(Counter)
    for p in pin.load():
        A = aa.get(p["a0"])
        if A is None:
            continue
        part = "hold" if pin.hold(p["a0"]) else "tune"
        for nm, L in lforms.forms(p, A).items():
            dz, miss, _ = lforms.judge_L(p, L)
            for key in ((nm, band(p["a0"]), part), (nm, "all", part)):
                tab[key]["dz"] += dz
                tab[key]["miss"] += miss
    print(f"{'form':8} {'a0 band':9} {'split':5} {'double-z cells':>15} {'misses':>7}")
    for key in sorted(tab):
        t = tab[key]
        print(f"{key[0]:8} {key[1]:9} {key[2]:5} {t['dz']:>15} {t['miss']:>7}")


if __name__ == "__main__":
    main()
