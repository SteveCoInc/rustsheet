"""The split model on r2-bpser-small's probes (t2464-r2-bpser-small-shapes: 2,000 shapes never tuned on; -runs: 14
x-runs at fixed shapes), every bpser sub-branch: cells where the GL53 kernel's only series call is bpser on the
cell's own (a, b, x) (no swap, so the cdf is that call's I). L is GL53's L of each sub-branch (b0 >= 8: r2-bratio's,
not solved). Counts by sub-branch and double / non-double z: z64 = r6's model, split = I = fl(fl(EXP(zh) s) fl(1+zl)).
'split with L53' is the split model with TOMS's gamln1 form of g (not run at b0 >= 8, whose L is unsolved).
usage: shapes.py"""
import json
from collections import Counter

import c7
import forms

cm = c7.cm
lib = cm.lib
L53 = lib.Model(lib.bx.BEST, lib.V(lg="L53"))  # TOMS's g = fl(gamln1(a0) - LN a0) in L (r2's L53)
R2 = c7.HERE.parent / "r2-bpser-small"


def main() -> None:
    cm.load_measured_gl()
    n = Counter()
    for nm in ("shapes", "runs"):
        out = json.load(open(R2 / f"t2464-r2-bpser-small-{nm}.out.json"))["sheets"]["Calc"]
        for i, _, x, a, b in json.load(open(R2 / f"t2464-r2-bpser-small-{nm}.meta.json")):
            c = out.get(f"A{i}")
            if not c or c.get("t") != "n":
                continue
            w, _, br, trace = lib.kernel(a, b, x, 1.0 - x, cm.GL53)
            if trace != [("bpser", a, b, x)] or not br.startswith("w_bpser"):
                continue
            sub = lib.sub_branch("bpser", a, b)
            cc = c7.cell(nm, a, b, x, float(c["v"]))
            f = forms.forms(cc)
            if f["z64"] != w:
                n[(nm, sub, "model differs from kernel", "cells")] += 1
                continue
            k = (nm, sub, "nd" if cc["nd"] else "dz")
            n[k + ("cells",)] += 1
            n[k + ("z64",)] += f["z64"] == cc["I"]
            n[k + ("split",)] += f[forms.KEY] == cc["I"]
            if sub != "bpser:b0>=8":
                c53 = c7.cell(nm, a, b, x, cc["I"], float(L53.lnb_w3(a, b)))
                n[k + ("split L53",)] += forms.forms(c53)[forms.KEY] == cc["I"]
    for k in sorted({k[:3] for k in n}):
        print(f"{k[0]:6s} {k[1]:13s} {k[2]}: cells {n[k + ('cells',)]:5d}, z64 exact {n[k + ('z64',)]:5d}, "
              f"split exact {n[k + ('split',)]:5d}, split with L53 " + (f"{n[k + ('split L53',)]:5d}" if "b0>=8" not in k[1] else "n/a"))


if __name__ == "__main__":
    main()
