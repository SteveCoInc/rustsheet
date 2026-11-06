"""T-4467's frozen literal-read model, as scored against the oracle (Train, Hold, the fresh sweep and the R4.2 sweep).

Exact rationals; RN53/RN64 are round-to-nearest-even to 53 and 64 significand bits, unbounded exponent (the probed
values are all normal). `frozen(m, place)` is Excel's read of the digits `m` (as written, after the 15-digit cut)
times 10^place for |place| >= 16; crates/rustsheet-engine/src/x87_table_read.rs implements it.
"""

from fractions import Fraction as F


def rnd(v: F, bits: int) -> F:
    if v == 0:
        return v
    s = -1 if v < 0 else 1
    v = abs(v)
    e = v.numerator.bit_length() - v.denominator.bit_length()
    if F(2) ** e > v:
        e -= 1
    if F(2) ** (e + 1) <= v:
        e += 1
    q = v / F(2) ** (e - bits + 1)
    n = q.numerator // q.denominator
    r = q - n
    if r > F(1, 2) or (r == F(1, 2) and n % 2):
        n += 1
    return s * F(n) * F(2) ** (e - bits + 1)


def table(e: int) -> F:
    """One table entry: 10^e rounded once to 64 bits."""
    return rnd(F(10) ** e, 64)


def frozen(m: int, place: int) -> F:
    """FROZEN 2026-11-06, before the hold-out was scored: n = 16q + r; r first, then 10^-16q (q <= 16), or
    10^-16(q-16) and then 10^-256."""
    n = abs(place)
    s = -1 if place < 0 else 1
    q, r = divmod(n, 16)
    v = F(m)
    if r:
        v = rnd(v * table(s * r), 64)
    if q:
        if q <= 16:
            v = rnd(v * table(s * 16 * q), 64)
        else:
            v = rnd(v * table(s * 16 * (q - 16)), 64)
            v = rnd(v * table(s * 256), 64)
    return rnd(v, 53)
