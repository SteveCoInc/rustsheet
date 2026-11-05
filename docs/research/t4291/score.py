"""T-4291: POISSON.DIST(k, mean, TRUE) far above the mean against two prefactor forms.

Q(k + 1, mean) = prefactor * fraction. The forms differ only in the prefactor's exponent, so each
prediction is the 50-digit Q times exp(E_form - E_true), E_form the form's double exponent:
- ratio form (T-2638, excel_ratio_exponent): a ln(x/a) + (a - x) above the mode;
- the accurate exponent (`rcomp`, which the engine takes): the 50-digit value.
Fit: probes/t3252-gamma-prefactor-window A97:A140; hold-out: probes/t4291-poisson-far-above-mean
(fixed before scoring). Ulps of Excel's value.
"""

import json
import math
import re
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
M = mp.mpf
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases"


def ratio_exponent(a, x):
    return (-a * math.log(a / x) if x < a else a * math.log(x / a)) + (a - x)


def rows(stem, refs=None):
    cells = json.loads((CASES / f"{stem}.case.json").read_text())["sheets"][0]["cells"]
    orc = json.loads((CASES / f"{stem}.oracle.json").read_text())["sheets"]["Calc"]
    for ref, f in cells.items():
        if refs and ref not in refs:
            continue
        m = re.fullmatch(r"=POISSON\.DIST\(([^,]+),([^,]+),TRUE\)", f)
        if m and ref in orc and orc[ref]["t"] == "n":
            yield ref, int(m[1]), float(m[2]), orc[ref]["v"]


def score(stem, refs=None):
    for ref, k, mean, xl in rows(stem, refs):
        if xl >= 0.5 or xl == 0:
            continue
        a, x = k + 1.0, mean
        q = mp.gammainc(M(a), M(x), mp.inf, regularized=True)
        e_true = M(a) * mp.log(M(x) / a) + a - M(x)
        ul = math.ulp(xl)
        r = q * mp.exp(M(ratio_exponent(a, x)) - e_true)
        print(f"{ref:5} k {k:4} mean/k {mean / k:5.2f}  accurate {float((q - xl) / ul):+8.1f}  ratio {float((r - xl) / ul):+8.1f}")




def summary(stem, refs=None):
    """Per form: cells within 2 ulp of Excel, rms ulp; and the mean |Excel's implied exponent - the
    form's| in ulps of E at the cells where one ulp of E spans at least 8 ulp of Q."""
    acc, rat, ea, er = [], [], [], []
    for _, k, mean, xl in rows(stem, refs):
        if xl >= 0.5 or xl == 0:
            continue
        a, x = k + 1.0, mean
        q = mp.gammainc(M(a), M(x), mp.inf, regularized=True)
        e_true = M(a) * mp.log(M(x) / a) + a - M(x)
        r = q * mp.exp(M(ratio_exponent(a, x)) - e_true)
        ul = math.ulp(xl)
        acc.append(float((q - xl) / ul))
        rat.append(float((r - xl) / ul))
        ue = math.ulp(float(e_true))
        if ue / (ul / xl) >= 8:
            implied = mp.log(M(xl) / q) + e_true
            ea.append(abs(float((implied - e_true) / ue)))
            er.append(abs(float((implied - M(ratio_exponent(a, x))) / ue)))
    for name, d in (("accurate", acc), ("ratio form", rat)):
        rms = math.sqrt(sum(v * v for v in d) / len(d))
        print(f"  {name:11} within 2 ulp {sum(abs(v) <= 2 for v in d):2}/{len(d)}, rms {rms:6.1f} ulp")
    print(f"  exponent gap, mean ulps of E over {len(ea)} cells: accurate {sum(ea) / len(ea):.2f}, ratio form {sum(er) / len(er):.2f}")


if __name__ == "__main__":
    print("fit: t3252-gamma-prefactor-window")
    score("t3252-gamma-prefactor-window", {f"A{r}" for r in range(97, 141)})
    print("hold-out: t4291-poisson-far-above-mean")
    score("t4291-poisson-far-above-mean")
    print("\nsummary, fit:")
    summary("t3252-gamma-prefactor-window", {f"A{r}" for r in range(97, 141)})
    print("summary, hold-out:")
    summary("t4291-poisson-far-above-mean")
