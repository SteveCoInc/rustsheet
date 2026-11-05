"""T-4350 (gap triage 2026-11-05): does Excel's large-shape GAMMA.DIST cdf carry its own density's error?

Usage: tools/.venv/bin/python docs/research/t4350/cdf_pairs.py  (needs mpmath; reads the t3252 cases and sidecars)

For every (x, a) of the t3252 probes that has both an Excel density and an Excel cdf at x/a 0.6 to 0.8 (scale 1),
it prints, in ulps of Excel's cdf:
- `excel-true`: Excel's cdf against the 50-digit regularized lower gamma P(a, x);
- `excel-pred`: Excel's cdf against Excel's own density f_E times (x / a) times the exact series
  S = P / (f x / a), the hypothesis that Excel's cdf is its density times an accurate series;
- `densErr`: Excel's density against the 50-digit density, in units of 2^-53 relative.
Nothing is fitted: the hypothesis has no free constant. Where `excel-pred` stays within a few ulp while
`excel-true` is tens to hundreds of ulp off, the cdf shares the density's prefactor.
"""

import json
import re
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
CASES = Path(__file__).resolve().parents[3] / "eval" / "suites" / "probes" / "cases"
NAMES = (
    "t3252-gamma-prefactor-holdout-2",
    "t3252-gamma-prefactor-sweep",
    "t3252-gamma-prefactor-holdout",
    "t3252-gamma-prefactor-window",
)
CALL = re.compile(r"=GAMMA\.DIST\(([^,]+),([^,]+),1,(TRUE|FALSE)\)")


def ulp(v: float) -> mp.mpf:
    """One unit in the last place of the double `v`."""
    return mp.mpf(2) ** (mp.floor(mp.log(abs(v), 2)) - 52)


def pairs(name: str) -> list[tuple[str, str, str, float, float]]:
    """(cdf cell, x, a, Excel density, Excel cdf) of every scale-1 pair in case `name`."""
    case = json.loads((CASES / f"{name}.case.json").read_text())
    oracle = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    dens: dict[tuple[str, str], float] = {}
    cdfs: dict[tuple[str, str], tuple[str, float]] = {}
    for cell, formula in case["sheets"][0]["cells"].items():
        m = CALL.match(formula)
        got = oracle.get(cell)
        if not m or not got or got["t"] != "n":
            continue
        if m.group(3) == "TRUE":
            cdfs[(m.group(1), m.group(2))] = (cell, got["v"])
        else:
            dens[(m.group(1), m.group(2))] = got["v"]
    out = []
    for key, (cell, cdf) in cdfs.items():
        if key in dens:
            out.append((cell, key[0], key[1], dens[key], cdf))
    return sorted(out, key=lambda p: (float(p[2]), float(p[1])))


def main() -> None:
    for name in NAMES:
        for cell, xs, as_, dens, cdf in pairs(name):
            x, a = mp.mpf(xs), mp.mpf(as_)
            ratio = float(x / a)
            if not 0.6 <= ratio <= 0.8:
                continue
            p = mp.gammainc(a, 0, x, regularized=True)
            f = mp.exp((a - 1) * mp.log(x) - x - mp.loggamma(a))
            pred = mp.mpf(dens) * x / a * (p / (f * x / a))
            u = ulp(cdf)
            print(
                f"{name.removeprefix('t3252-gamma-prefactor-'):10} {cell:4} a={as_:>8} x/a={ratio:.4f}",
                f"excel-true {float((mp.mpf(cdf) - p) / u):+8.1f}",
                f"excel-pred {float((mp.mpf(cdf) - pred) / u):+8.1f}",
                f"densErr {float((mp.mpf(dens) - f) / f * 2**53):+7.1f}",
            )


if __name__ == "__main__":
    main()
