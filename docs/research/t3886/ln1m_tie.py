"""T-3886: is Excel's ln(1 - x0) at the 6 shape-of-1 cells a near-tie of its log1p series (statistics.md "Excel's
log1p is a 14-term atanh series in alnrel's variable", whose 27 known 1-ulp misses are near-ties of r + s)?

Recomputes the series model (docs/research/t2464/r2-log1p/excel_log1p.py; every op x87, stored: fl53(fl64(.))) and
prints, for each cell's x0, the model, the correctly rounded value, and where the 64-bit r + s sits inside its 53-bit
ulp (0.5 = a tie of the final store), plus the same for the product b0 * l.
Usage (repo root): tools/.venv/bin/python docs/research/t3886/ln1m_tie.py
"""
import mpmath
from mpmath import floor, log, mp, mpf

mp.prec = 300

# (cell, x0, b0) after bratio's swap, as the engine's shape_one_tails receives them.
CELLS = [
    ("t2220-f-dist-sweep-cdf A191 F.DIST(0.1,2,100,TRUE)", 0.0019960079840319364, 50.0),
    ("t2220-f-dist-sweep-cdf A195 / -rt A195 F.DIST(1.5,2,100)", 0.02912621359223301, 50.0),
    ("t2220-f-dist-sweep-d2-1 A16 F.DIST(0.000117,2,1,TRUE)", 0.00023394525680990645, 0.5),
    ("t1745-beta-inv-roundtrip G145 BETA.DIST(D145,100,1,TRUE)", 0.029513049607040043, 100.0),
]


def rnd(v, p):
    return mpf(mpmath.libmp.mpf_pos(mpf(v)._mpf_, p, "n"))


def st(v):
    return rnd(rnd(v, 64), 53)


def frac_of_ulp(v):
    ulp = mpf(2) ** (floor(log(abs(v), 2)) - 52)
    return float((abs(v) / ulp) % 1)


def series(a):
    a = mpf(a)
    r = st(a / rnd(a + 2, 53))
    t2 = st(r * r)
    power, s = r, None
    for k in range(14):
        power = st(power * t2)
        term = st(power * rnd(mpf(1) / (2 * k + 3), 53))
        s = term if s is None else st(s + term)
    return float(2 * st(r + s)), frac_of_ulp(rnd(r + s, 64))


for name, x0, b0 in CELLS:
    model, frac = series(-x0)
    cr = float(mpmath.log1p(-mpf(x0)))
    print(f"{name}\n  l model {model!r}  correctly rounded {cr!r}  r+s at {frac:.3f} of an ulp; "
          f"b0*l at {frac_of_ulp(mpf(b0) * mpf(model)):.3f}")
