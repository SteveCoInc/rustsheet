"""Excel's GAMMALN at 0.5 and its r5 neighbours (a0 by single ulps): error against the exact lnGamma in ulps. If 0.5
alone is correctly rounded while its neighbours carry the routine's usual ~1 ulp error, the worksheet GAMMALN
special-cases 0.5 and bpser's internal call (GAMMALN(0.5) + 1 ulp, gscan.out) does not.
usage: gl_half.py"""
import math

import mpmath

import common as cm

mpmath.mp.dps = 40


def main() -> None:
    cm.load_measured_gl()
    gl = cm.lib.bx._GL
    for c in (0.3, 0.5, 0.7):
        vs = sorted(v for v in gl if abs(v - c) < 1e-13)
        errs = [float((mpmath.mpf(gl[v]) - mpmath.loggamma(mpmath.mpf(v))) / math.ulp(gl[v])) for v in vs]
        at = errs[vs.index(c)] if c in vs else None
        near = [e for v, e in zip(vs, errs) if v != c]
        print(f"center {c}: GAMMALN error at the center {at:+.3f} ulp; neighbours n={len(near)} mean {sum(near) / len(near):+.3f}"
              f" min {min(near):+.3f} max {max(near):+.3f}; the 4 nearest: "
              + " ".join(f"{e:+.2f}" for v, e in sorted(zip(vs, errs), key=lambda t: abs(t[0] - c))[1:5]))


if __name__ == "__main__":
    main()
