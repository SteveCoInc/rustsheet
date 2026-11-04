"""T-3582 r6-bgrat-q: candidate Q(b, z) algorithms for the start of Excel's direct bgrat (b != 1/2), each in doubles
unless T says otherwise. All return (q, h) where h = Q / r is the continued fraction or series ratio itself and
r = z^b e^-z / Gamma(b); bgrat's j0 = q / r, so a model can take j0 = h directly ("hj") or through the rounded q
("qj").

- nr_gcf: Numerical Recipes gcf, modified Lentz (FPMIN 1e-30, stop |del - 1| < eps), the loop T-3571 found in
  Excel's bfrac.
- nr_gcf_steed / cephes_igamc: other evaluations of the same Legendre fraction (Cephes igamc's three-term
  recurrence with its big-number rescale).
"""
from __future__ import annotations

import math

import numpy as np

FPMIN = 1e-30


def nr_gcf(b, z, eps, T=np.float64):
    """NR gcf: Q(b, z) / r by modified Lentz on the even contraction b0 = z + 1 - b, a_i = -i (i - b), b_i += 2."""
    b, z = T(b), T(z)
    bb = z + T(1) - b
    c = T(1) / T(FPMIN)
    d = T(1) / bb
    h = d
    for i in range(1, 2001):
        an = -T(i) * (T(i) - b)
        bb += T(2)
        d = an * d + bb
        if abs(d) < FPMIN:
            d = T(FPMIN)
        c = bb + an / c
        if abs(c) < FPMIN:
            c = T(FPMIN)
        d = T(1) / d
        de = d * c
        h *= de
        if abs(de - T(1)) <= T(eps):
            break
    return h


def cephes_igamc(b, z, eps, T=np.float64):
    """Cephes igamc's fraction (Q / r), three-term recurrence with its 2^52 rescale."""
    b, z = T(b), T(z)
    big, biginv = T(4503599627370496.0), T(2.22044604925031308085e-16)
    y = T(1) - b
    zz = z + y + T(1)
    c = T(0)
    pkm2, qkm2 = T(1), z
    pkm1, qkm1 = z + T(1), zz * z
    ans = pkm1 / qkm1
    for _ in range(2000):
        c += T(1)
        y += T(1)
        zz += T(2)
        yc = y * c
        pk = pkm1 * zz - pkm2 * yc
        qk = qkm1 * zz - qkm2 * yc
        if qk != 0:
            r = pk / qk
            t = abs((ans - r) / r)
            ans = r
        else:
            t = T(1)
        pkm2, pkm1, qkm2, qkm1 = pkm1, pk, qkm1, qk
        if abs(pk) > big:
            pkm2 *= biginv
            pkm1 *= biginv
            qkm2 *= biginv
            qkm1 *= biginv
        if t <= T(eps):
            break
    # Cephes returns ans * r, so ans is Q / r.
    return ans


def r_of(b, z, lgam=None):
    """r = z^b e^-z / Gamma(b) in doubles, as exp(b ln z - z - lnGamma(b)) (lgam: a lnGamma(b) override)."""
    lg = math.lgamma(b) if lgam is None else lgam
    return math.exp(b * math.log(z) - z - lg)
