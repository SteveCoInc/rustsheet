"""T-4308 (second PR): read Excel's large-shape gamma window exponent B as a double, then score candidate forms.

Usage: tools/.venv/bin/python docs/research/t4308/read_out.py  (needs mpmath; reads the cases and their oracle sidecars)

Part 1, the read-out. Inside the window (|x - a| <= 0.2 (x + a)) the density is the T-4296 chain
exp(-c(a) - B) / sqrt(2 pi a) * (a / x), c(a) the exact Stirling error. For each probed cell this
searches the doubles within 128 ulp of the true B = a ln(a/x) + x - a for the ones whose chain, in
doubles, is Excel's value bit for bit. One B is found per cell, or none: Excel's exponent is then
known exactly, where T-4255 and T-4290 could only read it as a per-unit error of +-0.5 ulp(ln u).
Part 2, the families, scored on that B (the split and the margin were fixed before scoring, see
the task file): the count of cells where a candidate's B is Excel's exactly, then the rms of its
distance in ulp(B). Fit = t4290 columns A:C, hold-out = column D.
Part 3, the fresh sweeps (decision 0188 R4.2 and R2): the engine's values (`rustsheet-eval show`, from a
release build) against Excel and the 50-digit density on probes-numeric/t4296-gamma-density-r4-fresh and
t4308-gamma-density-r4-fresh (written after the freeze): bit-exact / within 4 ulp / within 15 digits, for the
engine and the correctly rounded density, and the cells outside Gate A's envelope (neither within 1E-9
relative of the math nor within 15 digits of Excel).
"""

import importlib.util
import json
import math
import re
import subprocess
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "t4255"))
# t4290's scorer is also named score.py, so it is loaded by path under its own name.
_SPEC = importlib.util.spec_from_file_location("t4290_score", HERE.parent / "t4290" / "score.py")
t4290 = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(t4290)
from score_forms import atanh_series, bd0, toms_rlog  # importable once the paths above are set

mp.mp.dps = 50
M = mp.mpf
TWO_PI = 2 * math.pi
SEARCH = 128


def corr(a):
    return float(t4290.corr(a))


def chain(b, a, x, c):
    """The T-4296 window chain in doubles."""
    return math.exp(-c - b) / math.sqrt(TWO_PI * a) * (a / x)


def true_b(x, a):
    X, A = M(x), M(a)
    return A * mp.log(A / X) + X - A


def excel_b(x, a, excel):
    """The doubles B within SEARCH ulp of the true B whose chain is `excel` bit for bit."""
    c, b0 = corr(a), float(true_b(x, a))
    found, b = [], b0
    for _ in range(SEARCH):
        b = math.nextafter(b, -math.inf)
    for _ in range(2 * SEARCH + 1):
        if chain(b, a, x, c) == excel:
            found.append(b)
        b = math.nextafter(b, math.inf)
    return found


def series_w(w):
    """u - 1 - ln u = w - log1p(w) by its Taylor series in w = u - 1, summed until it stops moving."""
    term, acc, k = -w, 0.0, 2
    while k < 2000:
        term = -term * w
        nxt = acc + term / k
        if nxt == acc:
            break
        acc, k = nxt, k + 1
    return acc


def two_sum(p, q):
    s = p + q
    bp = s - p
    return s, (p - (s - bp)) + (q - bp)


def split(p):
    t = p * 134217729.0
    hi = t - (t - p)
    return hi, p - hi


def two_prod(p, q):
    """Dekker's exact product p q = hi + lo (no fma in this Python)."""
    hi = p * q
    ph, pl = split(p)
    qh, ql = split(q)
    return hi, ((ph * qh - hi) + ph * ql + pl * qh) + pl * ql


def fma_form(x, a):
    """fl(-a ln u + (x - a)) with the product kept exact (one rounding, as an fma would give)."""
    hi, lo = two_prod(-a, math.log(x / a))
    s, e = two_sum(hi, x - a)
    return s + (e + lo)


def candidates(x, a):
    """Each candidate B (density = exp(-c - B) / sqrt(2 pi a) (a / x)), in doubles."""
    u, w, v = x / a, (x - a) / a, (x - a) / (x + a)
    return {
        "correctly rounded B": float(true_b(x, a)),
        "a ln(a/x) + (x - a) (Loader's direct)": a * math.log(a / x) + (x - a),
        "-a ln(x/a) + (x - a)": -a * math.log(u) + (x - a),
        "fl(-a ln u + (x - a)), one rounding": fma_form(x, a),
        "-a log1p(w) + (x - a)": -a * math.log1p(w) + (x - a),
        "a((u - 1) - ln u)": a * ((u - 1) - math.log(u)),
        "a(w - log1p w)": a * (w - math.log1p(w)),
        "a fl(u - 1 - ln u) (B/a rounded)": a * float(M(u) - 1 - mp.log(u)),
        "a series_w(w)": a * series_w(w),
        "a TOMS 654 rlog(u)": a * toms_rlog(u),
        "bd0(a, x) (Loader's series)": bd0(a, x),
        "-2a atanh(v) + (x - a), series": -2 * a * atanh_series(v) + (x - a),
        "-2a atanh(v) + (x - a), libm": -2 * a * math.atanh(v) + (x - a),
        "(x - a) v - 2a(atanh(v) - v)": (x - a) * v - 2 * a * (math.atanh(v) - v),
        "a ln(a/x) - (a - x)": a * math.log(a / x) - (a - x),
        "(a ln(a/x) + x) - a": (a * math.log(a / x) + x) - a,
        "(x - a) v - 2a(atanh(v) - v), atanh rounded": (x - a) * v - 2 * a * (atanh_rounded(v) - v),
        "(x - a) v - (2a atanh(v) - 2a v), rounded": window_b(a, x),
    }


def atanh_rounded(v):
    return float(mp.atanh(M(v)))


def window_b(a, y):
    """The adopted form (density.rs `window_exponent`): Loader's bd0 split, atanh correctly rounded."""
    v = (y - a) / (y + a)
    return (y - a) * v - (2 * a * atanh_rounded(v) - 2 * a * v)


def direct_b(a, y):
    """Loader's direct deviance outside the window (density.rs `direct_deviance`)."""
    return a * math.log(a / y) + (y - a)


def stirling_series(z):
    """incomplete_beta.rs `stirling_correction`, the 8-term series the prefactor takes."""
    coeffs = [1 / 12, -1 / 360, 1 / 1260, -1 / 1680, 1 / 1188, -691 / 360360, 1 / 156, -3617 / 122400]
    w, acc = 1 / (z * z), coeffs[-1]
    for c in reversed(coeffs[:-1]):
        acc = acc * w + c
    return acc / z


def points():
    """(part, ref, x, a, Excel's B) for the t4290 window cells, and the refs with no B."""
    pts, missing = [], []
    for r in range(1, 227):
        for col in "ABCD":
            x, a, xl = t4290.cell(f"{col}{r}")
            if not (0.6668 < x / a < 1.4999) or abs(x / a - 1) < 0.02:
                continue
            found = excel_b(x, a, float(xl))
            if len(found) == 1:
                pts.append(("hold-out" if col == "D" else "fit", f"{col}{r}", x, a, found[0]))
            else:
                missing.append((f"{col}{r}", len(found)))
    return pts, missing


def part1(pts, missing):
    print("Part 1: Excel's B read out as a double on the T-4296 chain")
    for part in ("fit", "hold-out"):
        print(f"  {part}: {sum(p[0] == part for p in pts)} cells with exactly one B")
    print(f"  cells with no B or more than one: {missing}")
    print("  distance of Excel's B from the true B, in ulp(B), and in a ulp(ln u) (x/a, every 8th fit cell):")
    for _, ref, x, a, b in [p for p in pts if p[0] == "fit"][::8]:
        t = true_b(x, a)
        print(
            f"    {ref:5} {x / a:.6f} {float((b - t) / math.ulp(b)):+8.2f}"
            f" {float((b - t) / (a * math.ulp(math.log(x / a)))):+.3f}"
        )


def part2(pts):
    print("\nPart 2: families, cells with Excel's B exactly and rms ulp(B) (fit A:C | hold-out D)")
    names = list(candidates(30.0, 50.0))
    rows = []
    for name in names:
        out = []
        for part in ("fit", "hold-out"):
            ps = [p for p in pts if p[0] == part]
            errs = [(candidates(x, a)[name] - b) / math.ulp(b) for _, _, x, a, b in ps]
            exact = sum(e == 0 for e in errs)
            out.append((exact, len(ps), math.sqrt(sum(e * e for e in errs) / len(errs))))
        rows.append((name, out))
    for name, out in rows:
        cols = " | ".join(f"{e:3}/{n:3} ({100 * e / n:4.1f}%) {r:6.2f}" for e, n, r in out)
        print(f"  {name:40} {cols}")


def all_densities():
    """(case, x, a, y, Excel) for every probed large-shape GAMMA.DIST density with an oracle value."""
    num = r"(-?[0-9.]+(?:E[-+]?[0-9]+)?)"
    pat = re.compile(rf"=(?:GAMMA\.DIST|GAMMADIST)\({num},{num},{num},(?:FALSE|0)\)")
    for path in sorted((ROOT / "eval/suites").glob("*/cases/*.case.json")):
        sidecar = path.with_name(path.name.replace(".case.json", ".oracle.json"))
        if not sidecar.exists():
            continue
        oracle = json.loads(sidecar.read_text())["sheets"]
        for sheet in json.loads(path.read_text())["sheets"]:
            values = oracle.get(sheet["name"], {})
            for ref, formula in sheet.get("cells", {}).items():
                m = pat.fullmatch(formula) if isinstance(formula, str) else None
                if not m or values.get(ref, {}).get("t") != "n" or values[ref]["v"] == 0:
                    continue
                x, a, b = t4290.literal(m[1]), float(m[2]), float(m[3])
                if a >= 20:
                    yield path.name[: -len(".case.json")], x, a, x / b, values[ref]["v"]


def part4():
    print("\nPart 4: every probed density at shapes 20 and up, bit-exact counts")
    groups = {}
    for case, x, a, y, excel in all_densities():
        if abs(y - a) <= 0.2 * (y + a):
            where = "window, at the mode" if y == a else "window"
        elif y >= 0.005 * a:
            where = "below the window" if y < a else "above the window"
        else:
            continue
        c = corr(a)
        old = math.sqrt(a / TWO_PI) * math.exp(t4290.ratio_form(y, a) - stirling_series(a)) / x
        cr = chain(float(true_b(y, a)), a, x, c)
        new = chain(window_b(a, y) if where.startswith("window") else direct_b(a, y), a, x, c)
        g = groups.setdefault(where, [0, 0, 0, 0])
        g[0] += 1
        g[1] += new == excel
        g[2] += cr == excel
        g[3] += old == excel
    print("  region: cells, adopted chain, chain on the correctly rounded B, the ratio prefactor")
    for where, (n, new, cr, old) in groups.items():
        print(f"  {where:20} {n:5} {new:5} {cr:5} {old:5}")


ROOT = HERE.parents[2]
SHOW = re.compile(r"(Calc!A\d+) .*\n\s+formula : (.*)\n\s+expected: (.*)\n\s+actual  : (.*)")
NUM = r"(-?[0-9.]+(?:E[-+]?[0-9]+)?)"


def true_density(x, a, scale):
    X, A, S = M(t4290.literal(x)), M(a), M(scale)
    y = X / S
    return mp.exp((A - 1) * mp.log(y) - y - mp.loggamma(A)) / S


def tally(stem):
    out = subprocess.run(
        [str(ROOT / "target/release/rustsheet-eval"), "show", "--all", f"probes-numeric/{stem}.case.json"],
        capture_output=True, text=True, check=True, cwd=ROOT,
    ).stdout
    counts = {"engine": [0, 0, 0], "correctly rounded": [0, 0, 0]}
    outside, n = [], 0
    for ref, formula, expected, actual in SHOW.findall(out):
        m = re.fullmatch(rf"=GAMMA\.DIST\({NUM},{NUM},{NUM},FALSE\)", formula)
        try:
            excel = float(expected)
        except ValueError:
            continue
        if not m or excel == 0:
            continue
        n += 1
        truth = true_density(m[1], float(m[2]), float(m[3]))
        values = {"correctly rounded": float(truth)}
        try:
            values["engine"] = float(actual)
        except ValueError:
            values["engine"] = math.nan
        for name, v in values.items():
            c = counts[name]
            c[0] += v == excel
            c[1] += abs(v - excel) <= 4 * math.ulp(excel)
            c[2] += f"{v:.15g}" == f"{excel:.15g}"
        v = values["engine"]
        if not (abs(M(v) - truth) <= M("1e-9") * abs(truth) or f"{v:.15g}" == f"{excel:.15g}"):
            outside.append(ref)
    print(f"  {stem}: {n} numeric cells with a nonzero Excel value")
    for name, (exact, four, fifteen) in counts.items():
        print(f"    {name:18} bit-exact {exact:3}, within 4 ulp {four:3}, within 15 digits {fifteen:3}")
    print(f"    outside Gate A's envelope: {outside or 'none'}")


def part3():
    print("\nPart 3: the fresh sweeps")
    for stem in ("t4296-gamma-density-r4-fresh", "t4308-gamma-density-r4-fresh"):
        tally(stem)


if __name__ == "__main__":
    PTS, MISSING = points()
    part1(PTS, MISSING)
    part2(PTS)
    part3()
    part4()
