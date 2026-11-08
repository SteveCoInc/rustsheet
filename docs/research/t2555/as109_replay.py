"""T-2555 attempt 1: replay AS 109 (R's old qbeta, AS R19's acu) in doubles on an accurate CDF.

Run with mpmath: cd tools && uv run python ../docs/research/t2555/as109_replay.py (about a minute).
Prints, for the 8 tiny-quantile BETA.INV cells of probes-numeric/t1538-f-inv-huge-df, each Newton iterate's
relative distance from Excel's value (ln B exact, ln(1-x) as log(1-x) and as log1p), then the constant offset
delta of ln B that would put iterate 2 on Excel's value. Findings (task file, "Attempts"): no stop rule over these
iterates gives Excel's values, and the delta differs by three orders of magnitude between cells of one shape pair
(A133 -1.6E-8, A137 +3.3E-5, A145 +1.0E-3 at (2.5, 5E9)), so a constant ln B error is not the cause either.
"""

import math

import mpmath

mpmath.mp.dps = 60


def cdf(x, a, b):
    return float(mpmath.betainc(a, b, 0, x, regularized=True))


def lbeta(a, b):
    return math.lgamma(a) + math.lgamma(b) - math.lgamma(a + b)


def lbeta_mp(a, b):
    return float(mpmath.log(mpmath.beta(a, b)))


def start(a, pp, qq, beta):
    """AS 109's starting value for I_x(pp, qq) = a, a <= 1/2 (shared with docs/research/t4865)."""
    r = math.sqrt(-math.log(a * a))
    y = r - (2.30753 + 0.27061 * r) / (1 + (0.99229 + 0.04481 * r) * r)
    if pp > 1 and qq > 1:
        r = (y * y - 3) / 6
        s = 1 / (pp + pp - 1)
        t = 1 / (qq + qq - 1)
        h = 2 / (s + t)
        w = y * math.sqrt(h + r) / h - (t - s) * (r + 5 / 6 - 2 / (3 * h))
        x = pp / (pp + qq * math.exp(w + w))
    else:
        r = qq + qq
        t = 1 / (9 * qq)
        t = r * (1 - t + y * math.sqrt(t)) ** 3
        if t <= 0:
            x = 1 - math.exp((math.log1p(-a) + math.log(qq) + beta) / qq)
        else:
            t = (4 * pp + r - 2) / t
            if t <= 1:
                x = math.exp((math.log(a * pp) + beta) / pp)
            else:
                x = 1 - 2 / (t + 1)
    return x


def as109(
    alpha, p, q, acu_mode, ret_old=True, lb=lbeta_mp, fpu=1e-300, iters=1000, l1p=False
):
    beta = lb(p, q)
    a = alpha
    pp = p
    qq = q
    swap = False
    if alpha > 0.5:
        a = 1 - alpha
        pp = q
        qq = p
        swap = True
    x = start(a, pp, qq, beta)
    r = 1 - pp
    t = 1 - qq
    yprev = 0
    adj = 1
    if acu_mode == "r19":
        acu = max(fpu, 10 ** (-13 - 2.5 / (pp * pp) - 0.5 / (a * a)))
    else:
        acu = acu_mode
    xs = [x]
    for _ in range(iters):
        yy = cdf(x, pp, qq)
        yy = (yy - a) * math.exp(
            beta + r * math.log(x) + t * (math.log1p(-x) if l1p else math.log(1 - x))
        )
        if yy * yprev <= 0:
            prev = max(abs(adj), fpu)
        g = 1
        while True:
            adj = g * yy
            if abs(adj) < prev:
                tx = x - adj
                if 0 <= tx <= 1:
                    if prev <= acu or abs(yy) <= acu:
                        res = x if ret_old else tx
                        return (1 - res if swap else res), xs
                    if tx != 0 and tx != 1:
                        break
            g /= 3
        if tx == x:
            return (1 - x if swap else x), xs
        x = tx
        yprev = yy
        xs.append(x)
    return None, xs


cells = [
    ("A133", 0.5, 2.5, 5e9, 4.351460182692426e-10),
    ("A135", 0.5, 3, 5e9, 5.348120623898952e-10),
    ("A137", 0.025, 2.5, 5e9, 8.312123853710714e-11),
    ("A139", 0.025, 3, 5e9, 1.237344696832885e-10),
    ("A145", 1e-6, 2.5, 5e9, 1.2896160204554348e-12),
    ("A147", 1e-6, 3, 5e9, 3.650856591919023e-12),
    ("A153", 0.3, 2.5, 5e9, 2.9999080989808717e-10),
    ("A155", 0.3, 3, 5e9, 3.827551626637142e-10),
]

if __name__ == "__main__":
    for c, p, a, b, ex in cells:
        for l1p in (False, True):
            v, xs = as109(p, a, b, 1e-300, True, l1p=l1p)
            print(
                c, "log1p" if l1p else "log(1-x)", [f"{(x - ex) / ex:+.1e}" for x in xs]
            )


def iterate_k(p, a, b, delta, k):
    """Iterate k of the replay with ln B offset by `delta` (ln(1-x) as log1p), or NaN."""
    _, xs = as109(
        p, a, b, 1e-300, True, lb=lambda s, t: lbeta_mp(s, t) + delta, l1p=True
    )
    return xs[k] if len(xs) > k else float("nan")


if __name__ == "__main__":
    for c, p, a, b, ex in cells:
        lo, hi = -1e-3, 1e-3
        flo, fhi = iterate_k(p, a, b, lo, 2) - ex, iterate_k(p, a, b, hi, 2) - ex
        if flo * fhi > 0:
            print(c, "iterate 2: no delta in [-1E-3, 1E-3] reaches Excel")
            continue
        for _ in range(60):
            m = (lo + hi) / 2
            fm = iterate_k(p, a, b, m, 2) - ex
            if fm * flo > 0:
                lo, flo = m, fm
            else:
                hi = m
        print(c, "iterate 2 needs delta", lo)
