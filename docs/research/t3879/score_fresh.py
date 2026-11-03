"""T-3879 R4.2: score the fresh sweep t3879-bpser-split-fresh (gen_fresh.py), filled from excel-graph after the freeze.

Excel's GAMMALN at every argument of L is read from the probe's own 'gl' rows and loaded into the GL53 model. Per
sub-branch and double / non-double z: exact / within 4 ulp / within 15 digits (r2-bratio score_lib's ulpd and w15) of
the split model (T-3873's KEY form, unchanged), the unsplit GL53 (z64, today's engine form) and the correctly rounded
value (CR: mpmath betainc at 50 digits). Gate A envelope (decision 0188 R2): within 15 digits of Excel, or within 1e-9
relative of the 50-digit value (floor 1e-300); every point outside is listed. Cells where the model's z64 differs from
r2-bratio's kernel are counted and left out (as r7's shapes.py does).
usage: score_fresh.py   (bindings/python/.venv/bin/python, from this directory)
"""

from __future__ import annotations

import json
import sys
from collections import Counter

import gen_fresh as gf
import mpmath

sys.path.insert(0, str(gf.HERE.parent / "t2464" / "r7-bpser-nondouble-z"))
import forms  # noqa: E402

c7, cm, lib = gf.c7, gf.cm, gf.lib
sl = lib.sl


def main() -> None:
    out = json.load(open(lib.CASES / f"{gf.NAME}.oracle.json"))["sheets"]["Calc"]
    meta = json.load(open(gf.HERE / f"{gf.NAME}.meta.json"))
    val = {i: out.get(f"A{i}") for i, *_ in meta}
    gl = {v: float(val[i]["v"]) for i, k, v, _, _ in meta if k == "gl" and val[i] and val[i].get("t") == "n"}
    print(f"gl rows: {sum(k == 'gl' for _, k, *_ in meta)}, numeric {len(gl)}")
    lib.bx._GL.update(gl)
    n: Counter = Counter()
    outside = []
    for i, k, x, a, b in meta:
        if k != "beta":
            continue
        c = val[i]
        if not c or c.get("t") != "n":
            n["not a number"] += 1
            continue
        excel = float(c["v"])
        w = lib.kernel(a, b, x, 1.0 - x, cm.GL53)[0]
        cc = c7.cell("fresh", a, b, x, excel)
        f = forms.forms(cc)
        if f["z64"] != w:
            n["model differs from kernel"] += 1
            continue
        with mpmath.workdps(50):
            exact = mpmath.betainc(a, b, 0, x, regularized=True)
        cr = float(exact)
        for key in (lib.sub_branch("bpser", a, b), "all"):
            for kind in ("nd" if cc["nd"] else "dz", "all"):
                g = (key, kind)
                n[g + ("cells",)] += 1
                for nm, v in (("split", f[forms.KEY]), ("z64", f["z64"]), ("CR", cr)):
                    n[g + (nm, "exact")] += v == excel
                    n[g + (nm, "4ulp")] += sl.ulpd(v, excel) <= 4
                    n[g + (nm, "15d")] += sl.w15(v, excel)
        split = f[forms.KEY]
        rel = abs((mpmath.mpf(split) - exact) / max(abs(exact), mpmath.mpf("1e-300")))
        inside = sl.w15(split, excel) or rel <= mpmath.mpf("1e-9")
        n["envelope cells"] += 1
        n["envelope inside"] += inside
        if not inside:
            outside.append(f"A{i} BETA.DIST({x!r}, {a!r}, {b!r}) excel={excel!r} split={split!r} rel={float(rel):.3g}")
    for key in ("not a number", "model differs from kernel"):
        print(f"{key}: {n[key]}")
    print("sub-branch      z     cells | split exact/4ulp/15d | GL53 z64 exact/4ulp/15d | CR exact/4ulp/15d")
    for key in ("bpser:b0<=1", "bpser:1<b0<8", "all"):
        for kind in ("dz", "nd", "all"):
            g = (key, kind)
            row = " | ".join("/".join(f"{n[g + (nm, m)]:,}" for m in ("exact", "4ulp", "15d")) for nm in ("split", "z64", "CR"))
            print(f"{key:15s} {kind:4s} {n[g + ('cells',)]:6,} | {row}")
    print(f"Gate A envelope (split): {n['envelope inside']:,} of {n['envelope cells']:,} inside")
    for line in outside:
        print("outside", line)


if __name__ == "__main__":
    main()
