"""T-4434: the regularized incomplete beta at one huge shape, to 60 significant digits or more, by mpmath.

`mp.betainc` is unreliable here: it returns 0 or garbage for tails below about 1E-130 at a huge shape (T-4017's
reference.py). So this module sums DLMF 8.17.8 itself,

    I_z(p, q) = z^p (1 - z)^q / (p B(p, q)) * 2F1(p + q, 1; p + 1; z),

on the side where z is small (z = the small-shape argument). Every term of that 2F1 is positive, so the sum has no
cancellation: the ratio of consecutive terms is (p + q + n) z / (p + 1 + n), which grows while n < q z and then decays
geometrically. With q z near p (the regions of T-4434, both F and BETA.DIST) that is a few hundred to a few thousand
terms. The complementary tail is 1 minus the sum; it loses as many digits as it is small, so the working precision is
raised by -log10 of the tail and the value is kept only when two precisions agree to 60 digits.

Run with the tools venv (mpmath 1.3); imported by gen_probe.py, score.py and docs/research/t4435/.
"""

from __future__ import annotations

import mpmath as mp

DIGITS = 60


def _lower_series(p: mp.mpf, q: mp.mpf, z: mp.mpf) -> mp.mpf:
    """I_z(p, q) by DLMF 8.17.8 at the current precision; z must be below 1 (and is small in our use)."""
    log_pre = (
        p * mp.log(z) + q * mp.log1p(-z) - mp.log(p) - (mp.loggamma(p) + mp.loggamma(q) - mp.loggamma(p + q))
    )
    eps = mp.mpf(2) ** (-mp.mp.prec - 10)
    term = mp.mpf(1)
    total = mp.mpf(1)
    n = 0
    while True:
        term *= (p + q + n) * z / (p + 1 + n)
        total += term
        n += 1
        # Past the peak (n > q z) the ratio is below r = (p + q + n) z / (p + 1 + n) < 1, so the rest is below
        # term * r / (1 - r).
        r = (p + q + n) * z / (p + 1 + n)
        if r < 1 and term * r / (1 - r) < eps * total:
            break
        if n > 10_000_000:  # pragma: no cover - the regions here need at most a few thousand terms
            raise RuntimeError("series did not converge")
    return mp.exp(log_pre) * total


def _at(p, q, z_num, z_den, upper: bool, dps: int) -> mp.mpf:
    with mp.workdps(dps):
        z = mp.mpf(z_num) / mp.mpf(z_den)
        low = _lower_series(mp.mpf(p), mp.mpf(q), z)
        return +(1 - low) if upper else +low


def ibeta(p, q, z_num, z_den=1, upper: bool = False) -> mp.mpf:
    """I_z(p, q) (or 1 - I_z(p, q) when `upper`) with z = z_num / z_den taken exactly from the given doubles or
    integers, to at least DIGITS significant digits. z must be the small-side argument: z * q near p, not near 1."""
    dps = DIGITS + 30
    while True:
        first = _at(p, q, z_num, z_den, upper, dps)
        if first == 0:
            dps *= 2
            continue
        lost = max(0, int(-mp.log10(abs(first)))) if upper else 0
        a = _at(p, q, z_num, z_den, upper, dps + lost)
        b = _at(p, q, z_num, z_den, upper, dps + lost + 40)
        with mp.workdps(dps + lost + 40):
            if b != 0 and abs(a / b - 1) < mp.mpf(10) ** -(DIGITS + 5):
                return b
        dps += 60


def f_upper(x: float, d1: int, d2: int) -> mp.mpf:
    """F.DIST.RT(x, d1, d2) = 1 - I_u(d1/2, d2/2), u = d1 x / (d2 + d1 x) exact (not the rounded double)."""
    m = mp.mpf(d1) * mp.mpf(x)
    with mp.workdps(400):
        return ibeta(mp.mpf(d1) / 2, mp.mpf(d2) / 2, m, m + d2, upper=True)


def f_lower(x: float, d1: int, d2: int) -> mp.mpf:
    """F.DIST(x, d1, d2, TRUE) = I_u(d1/2, d2/2), u = d1 x / (d2 + d1 x) exact."""
    m = mp.mpf(d1) * mp.mpf(x)
    with mp.workdps(400):
        return ibeta(mp.mpf(d1) / 2, mp.mpf(d2) / 2, m, m + d2, upper=False)


def beta_lower(x: float, a: float, b: float) -> mp.mpf:
    """BETA.DIST(x, a, b, TRUE) = I_x(a, b) at the double x, a, b."""
    return ibeta(mp.mpf(a), mp.mpf(b), mp.mpf(x), 1, upper=False)


def rounded_ratio(x: float, d1: int, d2: int) -> float:
    """u = fl(fl(d1 x) / fl(fl(d1 x) + d2)), the small-side argument as a double kernel forms it (T-4017)."""
    m = d1 * x
    return m / (m + d2)


def f_upper_rounded(x: float, d1: int, d2: int) -> mp.mpf:
    """1 - I_u(d1/2, d2/2) at the double u of `rounded_ratio`: what a correctly rounded kernel would see."""
    return ibeta(mp.mpf(d1) / 2, mp.mpf(d2) / 2, mp.mpf(rounded_ratio(x, d1, d2)), 1, upper=True)


def f_lower_rounded(x: float, d1: int, d2: int) -> mp.mpf:
    """I_u(d1/2, d2/2) at the double u of `rounded_ratio`."""
    return ibeta(mp.mpf(d1) / 2, mp.mpf(d2) / 2, mp.mpf(rounded_ratio(x, d1, d2)), 1, upper=False)
