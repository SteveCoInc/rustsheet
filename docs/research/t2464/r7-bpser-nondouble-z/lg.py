"""T-3771's probe t3570-r5-bpser-lg (a0 stepped by single ulps at seven centers, three b0, 8 x; Excel's GAMMALN
columns beside the cells, read through r6-bpser-gpin's gpin.py): GL53 with E = EXP(extended z) against the split
model, split by double / non-double z, per center and by r5-bpser-lg's own tune / hold-out split (pin.hold).
usage: lg.py"""
import sys
from collections import Counter

import c7
import forms

sys.path.insert(0, str(c7.HERE.parent / "r6-bpser-gpin"))
import gpin  # noqa: E402

pin, sx = gpin.pin, c7.sx


def main() -> None:
    n = Counter()
    for p in pin.load():
        a0, b0 = p["a0"], p["b0"]
        L = sx("+", p["gl"], sx("-", p["glb"], p["glab"]))
        part = "hold" if pin.hold(a0) else "tune"
        for x, e in p["cells"]:
            if e is None:
                continue
            c = c7.cell(part, a0, b0, x, e, L)
            f = forms.forms(c)
            for k in ((f"{a0:.2f}", part, "nd" if c["nd"] else "dz"), ("all", part, "nd" if c["nd"] else "dz")):
                n[k + ("cells",)] += 1
                n[k + ("z64",)] += f["z64"] == e
                n[k + ("split",)] += f[forms.KEY] == e
    for k in sorted({k[:3] for k in n}):
        print(f"{k[0]:5s} {k[1]} {k[2]}: cells {n[k + ('cells',)]:5d}, z64 exact {n[k + ('z64',)]:5d}, "
              f"split exact {n[k + ('split',)]:5d}")


if __name__ == "__main__":
    main()
