"""T-3570's deferred re-score: the old w2-forensics BETA.DIST cdf cells (probes t2464-w2f-xruns-1/-2, absweep) at
a0 < 1 < b0 < 8 that the kernel sends to bpser directly (w_bpser), with Excel's measured GAMMALN loaded. Scores
GL53 with E = EXP(extended z) (r6's model) and the split model, split by double / non-double z and by whether
Excel's GAMMALN was measured at all three of a0, b0 and fl(a0 + b0) (else the engine's GAMMALN model stands in).
usage: w2f.py"""
import json
import re
from collections import Counter
from fractions import Fraction

import c7
import forms

cm = c7.cm
CASES = c7.HERE.parents[3] / "eval" / "suites" / "probes-numeric" / "cases"
NAMES = ("t2464-w2f-xruns-1", "t2464-w2f-xruns-2", "t2464-w2f-absweep")
CELL = re.compile(r"=BETA\.DIST\((.+),([0-9.]+),([0-9.]+),TRUE\)$")
TERM = re.compile(r"(\d+)\*2\^-(\d+)")


def xval(t: str) -> float:
    """The cell's x argument: a short decimal or the exact form (m*2^-e+n*2^-f) that w2f wrote."""
    if "*" not in t:
        return float(t)
    return float(sum(Fraction(int(m), 2 ** int(e)) for m, e in TERM.findall(t)))


def rows():
    for name in NAMES:
        case = json.load(open(CASES / f"{name}.case.json"))
        out = json.load(open(CASES / f"{name}.oracle.json"))["sheets"]["Calc"]
        for ref, f in case["sheets"][0]["cells"].items():
            m = CELL.match(str(f).replace(" ", ""))
            c = out.get(ref)
            if not m or not c or c.get("t") != "n":
                continue
            x, a, b = xval(m.group(1)), float(m.group(2)), float(m.group(3))
            if a < 1.0 < b < 8.0 and cm.lib.kernel(a, b, x, 1.0 - x, cm.GL53)[2].split("/")[0] == "w_bpser":
                yield name, b, a, x, float(c["v"])


def main() -> None:
    cm.load_measured_gl()
    rs = list(rows())
    shapes = sorted({(a, b) for _, b, a, _, _ in rs})
    gl = cm.lib.bx._GL
    meas = {(a, b): all(v in gl for v in (a, b, cm.sx("+", a, b))) for a, b in shapes}
    cells = c7.cells_from([(n, b, a, x, e) for n, b, a, x, e in rs])
    n = Counter()
    for c in cells:
        f = forms.forms(c)
        gm = "GAMMALN measured" if meas[(c["a"], c["b"])] else "GAMMALN(a0) modelled"
        k = (c["part"], gm, "nd" if c["nd"] else "dz")
        n[k + ("cells",)] += 1
        n[k + ("z64",)] += f["z64"] == c["I"]
        n[k + ("split",)] += f[forms.KEY] == c["I"]
    for k in sorted({k[:3] for k in n}):
        print(f"{k[0]} {k[1]} {k[2]}: cells {n[k + ('cells',)]}, z64 exact {n[k + ('z64',)]}, split exact "
              f"{n[k + ('split',)]}")


if __name__ == "__main__":
    main()
