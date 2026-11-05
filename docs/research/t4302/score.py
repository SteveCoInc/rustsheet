"""T-4302: POISSON.DIST's cdf far above the mean (k 30 to 300), scored against Excel.

Run from the repository root with the tools venv (mpmath), after `cargo build --release -p rustsheet_eval`:
    tools/.venv/bin/python docs/research/t4302/score.py

Splits, fixed before scoring (decision 0188 R4.1):
- train: probes/t4291-poisson-far-above-mean (the cdfs) with probes/t4302-poisson-cdf-beside-pmf rows 1:18
  (the same points' pmf, density and CHISQ.DIST.RT);
- hold-out: probes/t4302-poisson-cdf-beside-pmf rows 19:42 (48 points from seed 4302);
- fresh sweep after the freeze (R4.2): probes-numeric/t4302-poisson-cdf-r4-fresh (seed 43020).
Parts:
1. Excel's cdf against Excel's own gamma density times the exact x F (F = Q / (x^a e^-x / Gamma(a))).
2. Excel's density exponent against candidate forms of B, as ulps of the form's double.
3. The density chain exp(-c - B) / sqrt(2 pi a) * (a / x), bit-exact counts.
4. The engine's cdf (bit-exact / within 4 ulp / within 15 digits) and the correctly rounded Q, by split,
   with the Gate A envelope on the fresh sweep.
"""

import json
import math
import re
import subprocess
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
M = mp.mpf
ROOT = Path(__file__).resolve().parents[3]
L = math.log


def oracle(suite, stem):
    base = ROOT / "eval/suites" / suite / "cases"
    cells = json.loads((base / f"{stem}.case.json").read_text())["sheets"][0]["cells"]
    orc = json.loads((base / f"{stem}.oracle.json").read_text())["sheets"]["Calc"]
    return cells, orc


def args(f):
    return [float(v) for v in f.split("(", 1)[1].rstrip(")").split(",")[:2]]


def points():
    """(split, ref, k, mean, cdf, density) for every probed point."""
    out = []
    c1, o1 = oracle("probes", "t4291-poisson-far-above-mean")
    c2, o2 = oracle("probes", "t4302-poisson-cdf-beside-pmf")
    c3, o3 = oracle("probes-numeric", "t4302-poisson-cdf-r4-fresh")
    for col in "ABCDEFGH":
        for r in range(1, 7):
            k, m = args(c1[f"{col}{r}"])[:2]
            out.append(("train", f"{col}{r}", int(k), m, o1[f"{col}{r}"]["v"], o2[f"{col}{r + 6}"]["v"]))
        for r in range(19, 25):
            k, m = args(c2[f"{col}{r}"])[:2]
            out.append(("hold-out", f"{col}{r}", int(k), m, o2[f"{col}{r}"]["v"], o2[f"{col}{r + 12}"]["v"]))
        for r in range(1, 7):
            k, m = args(c3[f"{col}{r}"])[:2]
            out.append(("fresh", f"{col}{r}", int(k), m, o3[f"{col}{r}"]["v"], o3[f"{col}{r + 6}"]["v"]))
    return out


def exact(a, x):
    A, X = M(a), M(x)
    q = mp.gammainc(A, X, mp.inf, regularized=True)
    pref = mp.exp(A * mp.log(X) - X - mp.loggamma(A))
    return q, pref, q / pref


def stirlerr(a):
    return float(mp.loggamma(M(a)) - ((M(a) - 0.5) * mp.log(a) - a + mp.log(2 * mp.pi) / 2))


FORMS = {
    "a ln(a/x) + (x - a)  [Loader's direct bd0]": lambda a, x: a * L(a / x) + (x - a),
    "-(a ln(x/a) + (a - x))  [ratio form]": lambda a, x: -(a * L(x / a) + (a - x)),
    "(a ln(a/x) + x) - a": lambda a, x: (a * L(a / x) + x) - a,
    "-(a log1p((x - a)/a) + (a - x))": lambda a, x: -(a * math.log1p((x - a) / a) + (a - x)),
    "a((x/a - 1) - ln(x/a))": lambda a, x: a * ((x / a - 1) - L(x / a)),
    "a (ln a - ln x) + (x - a)": lambda a, x: a * (L(a) - L(x)) + (x - a),
}


def density_chain(a, x):
    b = a * L(a / x) + (x - a)
    return math.exp(-stirlerr(a) - b) / math.sqrt(2 * math.pi * a) * (a / x)


def engine_values():
    """The engine's value of every cell of the three cases, from `rustsheet-eval show`."""
    vals = {}
    binary = ROOT / "target/release/rustsheet-eval"
    for case in (
        "probes/t4291-poisson-far-above-mean.case.json",
        "probes/t4302-poisson-cdf-beside-pmf.case.json",
        "probes-numeric/t4302-poisson-cdf-r4-fresh.case.json",
    ):
        text = subprocess.run([binary, "--root", ROOT, "show", case], capture_output=True, text=True).stdout
        ref = None
        for line in text.splitlines():
            if m := re.match(r"^Calc!(\S+)", line):
                ref = m[1]
            elif (m := re.match(r"^\s+actual\s*: ?(.*)", line)) and ref:
                vals[(case.split("/")[1].split(".")[0], ref)] = float(m[1])
    return vals


def count(errs):
    return f"{sum(e == 0 for e in errs)}/{sum(abs(e) <= 4 for e in errs)}/{len(errs)}"


def main():
    pts = points()
    above = [p for p in pts if p[3] / (p[2] + 1) > 1.5]
    window = [p for p in pts if p[3] / (p[2] + 1) <= 1.5]
    cache = {(p[2], p[3]): exact(p[2] + 1.0, p[3]) for p in pts}

    print("1. Excel's cdf against Excel's density * x * F: exact/within 4 ulp/n")
    for name, sel in (("x/a > 1.5", above), ("x/a <= 1.5", window)):
        for split in ("train", "hold-out", "fresh"):
            errs = []
            for s, _, k, m, cdf, dens in sel:
                if s != split:
                    continue
                pred = float(M(dens) * M(m) * cache[(k, m)][2])
                errs.append((pred - cdf) / math.ulp(cdf))
            if errs:
                print(f"   {name:10} {split:8} {count(errs)}  max {max(map(abs, errs)):.0f} ulp")

    print("2. Excel's density exponent against each form of B, x/a > 1.5: within 1/2 ulp of the form / n, rms ulp")
    for name, form in FORMS.items():
        row = []
        for split in ("train", "hold-out", "fresh"):
            res = []
            for s, _, k, m, _, dens in above:
                if s != split:
                    continue
                a, x = k + 1.0, m
                A, X = M(a), M(x)
                true_dens = mp.exp((A - 1) * mp.log(X) - X - mp.loggamma(A))
                b_true = A * mp.log(A / X) + X - A
                b = form(a, x)
                res.append(float(-mp.log(M(dens) / true_dens) - (M(b) - b_true)) / math.ulp(b))
            rms = math.sqrt(sum(r * r for r in res) / len(res))
            row.append(f"{split} {sum(abs(r) <= 0.5 for r in res)}/{len(res)} rms {rms:.2f}")
        print(f"   {name:44} " + "; ".join(row))

    print("3. The density chain exp(-c - B)/sqrt(2 pi a)*(a/x), Loader's B: bit-exact/within 4 ulp/n")
    for name, sel in (("x/a > 1.5", above), ("x/a <= 1.5", window)):
        for split in ("train", "hold-out", "fresh"):
            errs = [
                (density_chain(k + 1.0, m) - d) / math.ulp(d) for s, _, k, m, _, d in sel if s == split
            ]
            if errs:
                print(f"   {name:10} {split:8} {count(errs)}")

    print("4. The cdf, x/a > 1.5: bit-exact/within 4 ulp/within 15 digits of n")
    eng = engine_values()
    stems = {"train": "t4291-poisson-far-above-mean", "hold-out": "t4302-poisson-cdf-beside-pmf",
             "fresh": "t4302-poisson-cdf-r4-fresh"}
    for split in ("train", "hold-out", "fresh"):
        for label in ("engine", "correctly rounded"):
            e = d15 = ex = n = 0
            outside = []
            for s, ref, k, m, cdf, _ in above:
                if s != split:
                    continue
                q = cache[(k, m)][0]
                v = eng.get((stems[split], ref), cdf) if label == "engine" else float(q)
                n += 1
                u = abs(v - cdf) / math.ulp(cdf)
                ex += u == 0
                e += u <= 4
                same15 = float(f"{v:.15g}") == float(f"{cdf:.15g}")
                d15 += same15
                if not (same15 or abs(M(v) / q - 1) <= M("1e-9")):
                    outside.append(ref)
            env = f"; Gate A envelope {n - len(outside)}/{n} inside" + (f", outside {outside}" if outside else "")
            print(f"   {split:8} {label:17} {ex}/{e}/{d15} of {n}" + (env if split == "fresh" else ""))


if __name__ == "__main__":
    main()
