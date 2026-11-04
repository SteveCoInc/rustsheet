"""Score T-3960's cases: ours vs Excel and Excel vs the 50-digit math, per family (statistics.md counts).

    rustsheet-eval show probes-numeric/CASE.case.json --all > SHOW.txt
    python score.py CASE.math.json SHOW.txt
"""

import json
import math
import re
import sys
from collections import defaultdict
from fractions import Fraction


def ulps(a: float, b: float) -> float:
    return abs(float(Fraction(a) - Fraction(b))) / math.ulp(b) if b else float("inf")


def binom_cdf(k: int, n: int, p):
    """P(X <= k) for X ~ Binomial(n, p) at mp.dps digits: the pmf summed outward from k (mpmath's betainc series does
    not converge at these sizes), the lower tail when k is at most the mean, else 1 minus the upper tail."""
    import mpmath as mp  # noqa: PLC0415

    q = 1 - p
    pmf = mp.exp(mp.loggamma(n + 1) - mp.loggamma(k + 1) - mp.loggamma(n - k + 1) + k * mp.log(p) + (n - k) * mp.log(q))
    eps = mp.mpf(10) ** -40
    term, j = pmf, k
    if k <= n * p:
        total = pmf
        while j > 0 and term > eps * total:
            term, j = term * j * q / ((n - j + 1) * p), j - 1
            total += term
        return total
    total = mp.mpf(0)
    while j < n and term > eps * (total or term):
        term, j = term * (n - j) * p / ((j + 1) * q), j + 1
        total += term
    return 1 - total


def reference_value(formula: str) -> Fraction | None:
    """50-digit-class value for a cdf cell the math sidecar skipped ("not converged or too slow"), by mpmath.

    the cdf as an outward pmf sum from k, since mpmath's betainc series does not converge at these sizes. p is the double nearest the literal, as Excel reads it and as the math sidecars use it.
    """
    import mpmath as mp  # noqa: PLC0415

    mp.mp.dps = 45
    g = re.match(r"=(\w+\.DIST)\((\d+),(\d+),([^,]+),TRUE\)", formula)
    if not g:
        return None
    fn, k, n, p = g[1], int(g[2]), int(g[3]), mp.mpf(float(g[4]))
    if fn == "BINOM.DIST":
        v = binom_cdf(k, n, p)
    else:
        # P(f failures or fewer before the r-th success) = P(at least r successes in f + r trials): r terms, not f.
        v = 1 - binom_cdf(n - 1, k + n, p)
    return Fraction(str(mp.nstr(v, 40)))


def within_fifteen_digits(expected: float, actual: float) -> bool:
    """The harness's rule (closeness.rs `within_fifteen_digits`, decision 0083): half a unit in the 15th digit."""
    if expected == 0.0:
        return actual == 0.0
    exponent = int(f"{abs(expected):.16e}".split("e")[1])  # the decimal exponent (never rounds up to the next power)
    return abs(actual - expected) <= float(f"5e{exponent - 15}")


def main(math_path: str, show_path: str) -> None:
    m = json.load(open(math_path))["sheets"]["Calc"]
    cells, cur = {}, None
    for line in open(show_path):
        if g := re.match(r"Calc!(A\d+)\s", line):
            cur = cells.setdefault(g[1], {})
        elif cur is not None and (g := re.match(r"\s+(formula|expected|actual)\s*: (.*)", line)):
            cur[g[1]] = g[2].strip()
    worst = []
    rows = defaultdict(lambda: defaultdict(int))
    for ref, c in cells.items():
        fam = "BINOM" if c["formula"].startswith("=BINOM.") else "NEGBINOM"
        try:
            x, o = float(c["expected"]), float(c["actual"])
            t = Fraction(m[ref]["v"]) if ref in m else reference_value(c["formula"])
            if ref in m:  # overlap check: our reference must agree with the sidecar
                alt = reference_value(c["formula"])
                assert abs(float(alt - t)) <= 1e-12 * abs(float(t)) or t == 0, ref
        except (ValueError, KeyError):
            rows[fam]["nonnumeric"] += 1
            continue
        r = rows[fam]
        r["cells"] += 1
        d = ulps(o, x)
        r["ours=excel bit-exact"] += d == 0
        r["ours within 4 ulp of excel"] += d <= 4
        r["ours within 15 digits of excel"] += within_fifteen_digits(x, o)
        e_math = abs(float(Fraction(x) - t)) / math.ulp(x) if x else 0
        o_math = abs(float(Fraction(o) - t)) / math.ulp(x) if x else 0
        r["excel correctly rounded (<= 0.5 ulp of math)"] += e_math <= 0.5
        worst.append((abs(float((Fraction(x) - t) / t)) if t else 0.0, ref, c["formula"]))
        r["excel within 4 ulp of math"] += e_math <= 4
        r["ours correctly rounded"] += o_math <= 0.5
        r["excel visibly off (> 100 ulp of math)"] += e_math > 100
        r["excel off and ours close (>100 / <=4)"] += e_math > 100 and o_math <= 4
    print("worst Excel relative error vs math:", max(worst))
    print(json.dumps({k: dict(v) for k, v in rows.items()}, indent=1))


main(*sys.argv[1:])
