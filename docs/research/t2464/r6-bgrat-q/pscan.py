"""T-3582 r6-bgrat-q pscan: which series is Excel's lower incomplete gamma P(b, z) at b < 1, small z? Scores
candidate P models in doubles and x87 extended on probe t3582-r6-gamma-runs (four runs of 1,024 consecutive doubles),
exact/w4/w15 per run, plus the sd of Excel's relative residual against the correctly rounded P (units of 2^-53) and
of Excel - model. A model whose residual sd falls well below CR's has Excel's arithmetic; one at or above it adds
its own noise."""
import math
import sys
from collections import defaultdict
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
from common import CASES, num, table  # noqa: E402  (r3-bgrat's readers, on the path above)
import json  # noqa: E402

import rustsheet as ic  # noqa: E402

NAME = "t3582-r6-gamma-runs"
LD = np.longdouble
EPS_D = 2.220446049250313e-16
mpmath.mp.dps = 40


def prefactor(b, z, T):
    """z^b e^-z / Gamma(b + 1) as exp(b ln z - z - lnGamma(b + 1)) in T (lnGamma exact, rounded to double)."""
    lg = T(float(mpmath.loggamma(mpmath.mpf(b) + 1)))
    return np.exp(T(b) * np.log(T(z)) - T(z) - lg)


def nr_gser(b, z, T, eps):
    """NR gser: sum 1/b + z/(b(b+1)) + ..., P = sum * exp(-z + b ln z - lnGamma(b))."""
    bt, zt = T(b), T(z)
    ap, de = bt, T(1) / bt
    s = de
    for _ in range(2000):
        ap += 1
        de *= zt / ap
        s += de
        if abs(de) < abs(s) * T(eps):
            break
    lg = T(float(mpmath.loggamma(b)))
    return float(s * np.exp(-zt + bt * np.log(zt) - lg))


def cephes_igam(b, z, T, eps):
    """Cephes igam: ans = 1 + z/(b+1) + ..., P = ans * z^b e^-z / Gamma(b) / b."""
    bt, zt = T(b), T(z)
    r, c, ans = bt, T(1), T(1)
    for _ in range(2000):
        r += 1
        c *= zt / r
        ans += c
        if c / ans <= T(eps):
            break
    lg = T(float(mpmath.loggamma(b)))
    return float(ans * np.exp(bt * np.log(zt) - zt - lg) / bt)


def toms_series(b, z, T, eps):
    """TOMS 708 grat1's z < 1.1 series in T: an = 3, c = z, sum = z/(b+3), ...; t = sum * b ... rolled up with
    gam1-free form P = z^b / Gamma(b + 1) * (1 - b z (1/(b+1) - z/(2(b+2)) + ...)) (alternating series)."""
    bt, zt = T(b), T(z)
    # alternating series of z^b/Gamma(b+1) * sum_k (-z)^k b / (k! (b + k))
    s, t = T(1), T(1)
    for k in range(1, 2000):
        t *= -zt / T(k)
        term = t * bt / (bt + T(k))
        s += term
        if abs(term) <= abs(s) * T(eps):
            break
    lg = T(float(mpmath.loggamma(mpmath.mpf(b) + 1)))
    return float(s * np.exp(bt * np.log(zt) - lg))


def engine(rows):
    m = ic.create("w", "en", "UTC")
    for i, (b, z) in enumerate(rows):
        m.update_cell_with_number(0, i + 1, 1, z)
        m.update_cell_with_number(0, i + 1, 2, b)
        m.update_cell_with_formula(0, i + 1, 3, f"=GAMMA.DIST(A{i + 1},B{i + 1},1,TRUE)")
    m.evaluate()
    out = []
    for i in range(len(rows)):
        v = m.get_cell_value(0, i + 1, 3) if hasattr(m, "get_cell_value") else None
        out.append(float(v[-1] if isinstance(v, tuple) else v))
    return out


MODELS = {
    "nr_gser d": lambda b, z: nr_gser(b, z, np.float64, EPS_D),
    "nr_gser ld": lambda b, z: nr_gser(b, z, LD, EPS_D),
    "cephes d": lambda b, z: cephes_igam(b, z, np.float64, EPS_D / 2),
    "cephes ld": lambda b, z: cephes_igam(b, z, LD, EPS_D / 2),
    "alt series d": lambda b, z: toms_series(b, z, np.float64, EPS_D / 2),
    "alt series ld": lambda b, z: toms_series(b, z, LD, EPS_D / 2),
}


def main():
    sh = json.load(open(CASES / f"{NAME}.oracle.json"))["sheets"]["Calc"]
    meta = json.load(open(HERE / f"{NAME}.meta.json"))
    rows = [(b, z, run, num(sh.get(f"A{k}"))) for k, b, z, run in meta]
    rows = [r for r in rows if r[3] is not None]
    eng = engine([(b, z) for b, z, _r, _e in rows])
    g = defaultdict(list)
    res = defaultdict(list)
    for (b, z, run, e), ev in zip(rows, eng):
        ex = mpmath.gammainc(b, 0, z, regularized=True)
        cr = float(ex)
        cand = {"CR": cr, "engine": ev, **{k: f(b, z) for k, f in MODELS.items()}}
        res[(run, "Excel vs exact")].append(float((mpmath.mpf(e) - ex) / ex) / 2 ** -53)
        for k, v in cand.items():
            g[(k, run)].append((v, e))
            g[(k, "ALL")].append((v, e))
            res[(run, f"Excel - {k}")].append((e - v) / e * 2.0 ** 53)
    table(g, ("model", "run"))
    print()
    for k, v in sorted(res.items(), key=str):
        v = np.array(v)
        print(k, f"mean {v.mean():+.2f} sd {v.std():.2f}")


if __name__ == "__main__":
    main()
