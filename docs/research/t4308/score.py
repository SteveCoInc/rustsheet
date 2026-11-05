"""T-4308: read the exponent B of Excel's large-shape gamma density inside the window, and check Gate A.

Usage, from the repository root after `cargo build --release -p rustsheet_eval`:
    tools/.venv/bin/python docs/research/t4308/score.py   (needs mpmath; RUSTSHEET_EVAL overrides the binary)

Part 1, the grid. On the T-4296 chain exp(-c(a) - B) / sqrt(2 pi a) (a / x), with c(a) the exact Stirling
error, Excel's implied B = -ln(density / prefactor) - c(a) is read to about 1E-16 absolute. Per column of
probes/t4290-gamma-window-large-pow2 the script prints the fractional part of B / g for the grid g = a 2^-59
(2^-47 at shape 4096 in A, 2^-49 at 1024 in B, 2^-48 at 2048 in D), and the spread of those parts once the
column's mean is removed. A spread near the read-out noise (0.01 to 0.06 of g) means the implied B sits on
that grid up to one constant per shape; a uniform spread is 0.29.

Part 2, families. Each candidate per-unit exponent L(u) (B = -a L + (x - a), u = x / a) against Excel's
implied L, as rms in ulp(ln u) per column A to D, then the count within 1/16 ulp. The split is t4290's,
fixed before scoring: fit = A, B, C; hold-out = D. Cells within 0.02 of the mode are left out.
A family the grid already rules out is still scored, so the record carries its number.

Part 3, Gate A (decision 0188 R2) on probes-numeric/t4296-gamma-density-r4-fresh, the T-4296 fresh sweep:
the engine (rustsheet-eval show) is inside the envelope when it is within 15 significant digits of Excel or
within 1E-9 relative of the 50-digit density. It prints the engine's bit-exact, within 4 ulp and within
15 digits counts and every point outside the envelope.
"""

import importlib.util
import math
import os
import re
import subprocess
import sys
from pathlib import Path

import mpmath as mp

ROOT = Path(__file__).resolve().parents[3]
# t4290's scorer is also named score.py: load it under its own name.
_SPEC = importlib.util.spec_from_file_location("t4290_score", ROOT / "docs/research/t4290/score.py")
t4290 = importlib.util.module_from_spec(_SPEC)
sys.path.insert(0, str(ROOT / "docs/research/t4255"))  # t4290 imports t4255's score_forms
_SPEC.loader.exec_module(t4290)
M, cell, corr = t4290.M, t4290.cell, t4290.corr

mp.mp.dps = 50
LN2 = mp.log(2)
FRESH = "probes-numeric/t4296-gamma-density-r4-fresh.case.json"


def implied(col):
    """(x, a, u, implied B, implied L) for every window cell of a t4290 column."""
    out = []
    for r in range(1, 227):
        x, a, xl = cell(f"{col}{r}")
        u = x / a
        if not 0.6668 < u < 1.4999:
            continue
        pref = 1 / mp.sqrt(2 * mp.pi * a) * (M(a) / x)
        b = -mp.log(xl / pref) - corr(a)
        out.append((x, a, u, b, (x - a - b) / a))
    return out


def part1():
    print("Part 1: implied B on the grid a 2^-59, per column (fractional parts, then spread about the mean)")
    for col in "ABCD":
        rows = [r for r in implied(col) if r[3] > 8]
        a = rows[0][1]
        g = M(a) * M(2) ** -59
        fr = [float(b / g - mp.nint(b / g)) for _, _, _, b, _ in rows]
        mean = math.atan2(
            sum(math.sin(2 * math.pi * f) for f in fr), sum(math.cos(2 * math.pi * f) for f in fr)
        )
        mean /= 2 * math.pi
        dev = [((f - mean + 0.5) % 1) - 0.5 for f in fr]
        rms = math.sqrt(sum(d * d for d in dev) / len(dev))
        noise = 2.0**-53 / float(g)  # one density ulp, as absolute error in B
        print(
            f"  {col} shape {a:6.0f}: {len(rows)} cells with B > 8, offset {mean:+.3f} g, spread {rms:.3f} g "
            f"(read-out noise about {noise:.3f} g)"
        )
    print("  B against its own double grid at shape 4096 (fractional part of B / (ulp(B) 2^-k), k = 0..3):")
    for x, a, u, b, _ in implied("A")[::12]:
        ub = math.ulp(float(b))
        fr = " ".join(f"{float(b / (ub * 2.0**-k) - mp.nint(b / (ub * 2.0**-k))):+.2f}" for k in range(4))
        print(f"    u {u:.5f} B {float(b):9.4f}  {fr}")


def split(v, lo):
    m, e = math.frexp(v)
    while m < lo:
        m, e = m * 2, e - 1
    while m >= 2 * lo:
        m, e = m / 2, e + 1
    return m, e


def fixed(v, k):
    g = M(2) ** -k
    return mp.nint(v / g) * g


def mantissa_log(lo, ln2):
    def f(x, a, u):
        mx, ex = split(x, lo)
        ma, ea = split(a, lo)
        return (ex - ea) * ln2 + M(math.log(mx)) - M(math.log(ma))

    return f


# TOMS 708's rlog1(w) = w - ln(1 + w) (Didonato and Morris), in doubles; constants as published.
RLOG1 = (0.0566749439387324, 0.0456512608815524, 0.333333333333333, -0.224696413112536, 0.00620886815375787,
         -1.27408923933623, 0.354508718369557)


def rlog1(w):
    a, b, p0, p1, p2, q1, q2 = RLOG1
    if w < -0.39 or w > 0.57:
        return w - math.log(w + 1.0)
    if w < -0.18:
        h = (w + 0.3) / 0.7
        w1 = a - h * 0.3
    elif w > 0.18:
        h = w * 0.75 - 0.25
        w1 = b + h / 3.0
    else:
        h, w1 = w, 0.0
    r = h / (h + 2.0)
    t = r * r
    q = ((p2 * t + p1) * t + p0) / ((q2 * t + q1) * t + 1.0)
    return t * 2.0 * (1.0 / (1.0 - r) - r * q) + w1


FAMILIES = {
    "accurate ln u (chance)": lambda x, a, u: mp.log(M(u)),
    "correctly rounded ln u": lambda x, a, u: M(math.log(u)),
    "log1p((x - a)/a)": lambda x, a, u: M(math.log1p((x - a) / a)),
    "ln u to the absolute grid 2^-55": lambda x, a, u: fixed(mp.log(M(u)), 55),
    "ln u to the absolute grid 2^-57": lambda x, a, u: fixed(mp.log(M(u)), 57),
    "ln x - ln a, each to the grid 2^-56": lambda x, a, u: fixed(mp.log(M(x)), 56) - fixed(mp.log(M(a)), 56),
    "ln x - ln a, each to the grid 2^-57": lambda x, a, u: fixed(mp.log(M(x)), 57) - fixed(mp.log(M(a)), 57),
    "(e_x - e_a) ln2 + ln m_x - ln m_a, m in [1/2, 1)": mantissa_log(0.5, LN2),
    "(e_x - e_a) ln2 + ln m_x - ln m_a, m in [1, 2)": mantissa_log(1.0, LN2),
    "(e_x - e_a) ln2 + ln m_x - ln m_a, m in [sqrt 1/2, sqrt 2)": mantissa_log(math.sqrt(0.5), LN2),
    "the same, ln2 as a double": mantissa_log(math.sqrt(0.5), M(math.log(2))),
    "(u - 1) - rlog1(u - 1), TOMS 708": lambda x, a, u: (u - 1) - M(rlog1(u - 1.0)),
}


def part2():
    print("\nPart 2: families, rms in ulp(ln u) and count within 1/16 ulp; fit A B C | hold-out D")
    cols = {c: [r for r in implied(c) if abs(r[2] - 1) >= 0.02] for c in "ABCD"}
    for name, f in FAMILIES.items():
        out = []
        for c in "ABCD":
            e = [float((lx - f(x, a, u)) / math.ulp(math.log(u))) for x, a, u, _, lx in cols[c]]
            out.append(f"{math.sqrt(sum(t * t for t in e) / len(e)):.2f} {sum(abs(t) < 1 / 16 for t in e):3}")
        print(f"  {name:58} {' '.join(out[:3])} | {out[3]}")


def within_15(v, excel):
    if excel == 0:
        return v == 0
    return abs(M(v) - M(excel)) <= M(10) ** (mp.floor(mp.log10(abs(M(excel)))) - 14) / 2


def ulps(v, excel):
    return abs(v - excel) / math.ulp(excel)


def part3():
    print(f"\nPart 3: Gate A on {FRESH}")
    shown = subprocess.run(
        [os.environ.get("RUSTSHEET_EVAL", str(ROOT / "target/release/rustsheet-eval")), "show", "--all", FRESH],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    ).stdout
    pat = re.compile(
        r"Calc!(A\d+)\s+\w+\n\s+formula : =GAMMA\.DIST\(([^,]+),([^,]+),([^,]+),FALSE\)\n"
        r"\s+expected: ([^\n]*)\n\s+actual  : ([^\n]*)"
    )
    counts = {"window": [0, 0, 0, 0], "x = a": [0, 0, 0, 0], "other": [0, 0, 0, 0]}
    inside, outside = 0, []
    for ref, xs, as_, bs, exp_s, act_s in pat.findall(shown):
        x, a, b = M(xs), M(as_), M(bs)
        excel, engine = float(exp_s), float(act_s)
        y = x / b
        truth = mp.exp((a - 1) * mp.log(y) - y - mp.loggamma(a)) / b
        kind = "x = a" if abs(y - a) <= mp.mpf("1e-12") * a else "window" if abs(y - a) <= mp.mpf("0.2") * (y + a) else "other"
        c = counts[kind]
        c[0] += 1
        c[1] += engine == excel
        c[2] += ulps(engine, excel) <= 4
        c[3] += within_15(engine, excel)
        ok = within_15(engine, excel) or abs(M(engine) - truth) <= max(
            mp.mpf("1e-9") * abs(truth), mp.mpf("1e-300")
        )
        inside += ok
        if not ok:
            outside.append(f"{ref} GAMMA.DIST({xs},{as_},{bs}) excel={excel!r} engine={engine!r}")
    for kind, (n, e, f4, d15) in counts.items():
        print(f"  {kind:7} {n:3} cells: bit-exact {e}, within 4 ulp {f4}, within 15 digits {d15}")
    print(f"  Gate A envelope: {inside} of {sum(c[0] for c in counts.values())} inside")
    for line in outside:
        print(f"  outside {line}")


if __name__ == "__main__":
    part1()
    part2()
    part3()
