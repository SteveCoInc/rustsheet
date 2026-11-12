"""T-5524: Excel's complement of p in CHISQ.INV.RT and CHIINV at one degree of freedom, three candidate forms,
and the exact lower quantile of a double at one degree of freedom.

`x87` is the rule: `(0.5 - p) + 0.5` with Excel's twice-rounded operators. The difference is rounded to a 64-bit
significand (an x87 register) and then stored to a double; the sum likewise. `plain` is `1 - p` in doubles (what
the engine used before), and `double` is `(0.5 - p) + 0.5` in doubles with no register stage, which differs from
`x87` only where the register rounding lands the difference on a tie of the store.
"""

import re
from fractions import Fraction

import mpmath as mp

from rustsheet_tools.fitlab.sweep import number_literal

mp.mp.dps = 60

HALF = Fraction(1, 2)
# One term of a sum of powers of two: `2^-53`, `-2^-66`, `+3*2^-55`.
TERM = re.compile(r"([+-]?)(?:(\d+)\*)?2\^(-?\d+)")
STEP = Fraction(1, 2**53)  # the spacing of the doubles below 1


def rounded(x: Fraction, bits: int) -> Fraction:
    """x > 0 rounded to `bits` significant bits, ties to even."""
    e = x.numerator.bit_length() - x.denominator.bit_length()
    if Fraction(2) ** e > x:
        e -= 1
    scale = Fraction(2) ** (e - bits + 1)
    m = x / scale
    whole = m.numerator // m.denominator
    rest = m - whole
    if rest > HALF or (rest == HALF and whole % 2 == 1):
        whole += 1
    return whole * scale


def twice(x: Fraction) -> Fraction:
    """One of Excel's `+` or `-`: the exact result rounded to 64 bits, then to 53 (the magnitude; ties to even)."""
    if x == 0:
        return x
    magnitude = rounded(rounded(abs(x), 64), 53)
    return magnitude if x > 0 else -magnitude


def x87(p: float) -> float:
    return float(twice(twice(HALF - Fraction(p)) + HALF))


def double(p: float) -> float:
    return (0.5 - p) + 0.5


def plain(p: float) -> float:
    return 1.0 - p


RULES = {"1 - p": plain, "(0.5 - p) + 0.5 in doubles": double, "(0.5 - p) + 0.5 twice-rounded": x87}


def steps(p: float) -> Fraction:
    """p in units of 2^-53, exactly."""
    return Fraction(p) / STEP


def quantile(u: float) -> float | None:
    """The correctly rounded x with erf(sqrt(x / 2)) = u, the lower chi-square quantile of the double u at one
    degree of freedom, found on the upper tail 1 - u (exact in 60 digits). None where u is not inside (0, 1)."""
    if not 0.0 < u < 1.0:
        return None
    tail = mp.mpf(1) - mp.mpf(u)
    s = mp.erfinv(mp.mpf(u))
    for _ in range(3):
        s -= (mp.erfc(s) - tail) / (-2 / mp.sqrt(mp.pi) * mp.exp(-s * s))
    return float(2 * s * s)


def argument(text: str) -> float:
    """The double a probe's p is: a number literal as Excel reads it (cut to 15 significant digits; the fit lab's
    `number_literal`), or a sum of terms `m*2^e` or `2^e` whose every partial sum is a double, so that Excel's
    arithmetic and exact arithmetic agree step by step. ValueError for anything else."""
    if "^" not in text:
        return number_literal(text)
    terms = list(TERM.finditer(text))
    if "".join(m.group(0) for m in terms) != text:
        raise ValueError(text)
    if terms[0].group(1) == "-":
        msg = f"{text}: a leading minus binds before ^ in Excel"
        raise ValueError(msg)
    total = Fraction(0)
    for term in terms:
        sign = -1 if term.group(1) == "-" else 1
        total += sign * int(term.group(2) or 1) * Fraction(2) ** int(term.group(3))
        if Fraction(float(total)) != total:
            msg = f"{text}: a partial sum is no double"
            raise ValueError(msg)
    return float(total)


def right_tail_quantile(p: float) -> float:
    """The correctly rounded x with erfc(sqrt(x / 2)) = p: the math's right-tail quantile of p itself, which keeps
    the digits a complement loses."""
    tail = mp.mpf(p)
    s = mp.sqrt(-mp.log(tail))
    for _ in range(60):
        step = (mp.erfc(s) - tail) / (-2 / mp.sqrt(mp.pi) * mp.exp(-s * s))
        s -= step
        if abs(step) < mp.mpf(10) ** -50:
            break
    return float(2 * s * s)
