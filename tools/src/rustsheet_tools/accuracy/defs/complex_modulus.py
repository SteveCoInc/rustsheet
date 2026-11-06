"""IMABS and IMARGUMENT: the modulus and the argument of a complex number's text (T-4085).

The math: |z| and arg(z) of the complex number whose parts are the doubles of the text's two written literals, at the
working precision. The text parameter is T-4084's complex kind (`Param(kind="text")`, no choices).

Sources:
- engineering.md, "IM* reciprocal kernels, IMABS and IMARGUMENT (T-2044)": Excel's IMABS is the scaled form
  `a·sqrt(1 + (b/a)²)` and IMARGUMENT is ATAN2's kernel, `atan(y/x)` plus or minus π. Those are Excel's kernels,
  not semantic rules, so the definitions are the true modulus and the true atan2(y, x) on (-π, π]; the envelope
  judges the kernels' last digits (T-2538).
- engineering.md, "IM* last-digit sweep: IMSEC and IMSECH use different kernels (T-2043)": `IMARGUMENT("0")` is
  `#DIV/0!` (probes-numeric/im-last-digit-sweep C21), ATAN2's `ATAN2(0,0)`: a domain error at z = 0.
- engineering.md, "IM* reciprocal kernels, IMABS and IMARGUMENT (T-2044)": the overflow and subnormal edges of the
  ATAN2 kernel are UNVERIFIED for IMARGUMENT. ATAN2 itself is `#NUM!` where y/x overflows (numbers.md "ATAN2 is
  ATAN(y/x) plus or minus π"), but that is not evidenced for IMARGUMENT, so such a point has no value here
  (NoValueError: the sampler redraws, an eval cell is skipped). A quotient that underflows moves the result by less
  than Gate A's absolute floor, as for ATAN2.
- engineering.md, "Leading plus in IM*, 11-digit OCT/HEX text and booleans in OCT2*/HEX2* (T-2420)": the texts
  `a+bi`, `a-bj`, `bi`, `i`, `-j` and a bare real are complex numbers; the edge stratum writes each form.
- A modulus beyond the double range cannot occur: a part is a literal below 1E+308, so |z| < 1.42E+308.
"""

from __future__ import annotations

import random
from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.literal import (
    COMPLEX_SUFFIXES,
    MAX_LITERAL,
    MIN_LITERAL,
    Arg,
    complex_text,
    round15,
)
from rustsheet_tools.accuracy.model import Definition, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.accuracy.tables import DOUBLE_MAX

KERNEL_SOURCE = "engineering.md#im-reciprocal-kernels-imabs-and-imargument-t-2044"
ZERO_SOURCE = 'probes-numeric/im-last-digit-sweep C21 (excel-graph): IMARGUMENT("0") is #DIV/0!'
GRAMMAR_SOURCE = "engineering.md#leading-plus-in-im-11-digit-octhex-text-and-booleans-in-oct2hex2-t-2420"

ZERO_TEXTS = ("0", "0+0i", "0-0j")
"""Zero in three spellings: IMARGUMENT's `#DIV/0!`, drawn for a quarter of the edge points."""

EDGE_TEXTS = (
    "1",
    "-7",
    "2.5i",
    "-3.25j",
    "i",
    "-j",
    "+i",
    "-1+0i",
    "-1+1E-300i",
    "-1-1E-300j",
    "3+4i",
    "-3-4j",
    "1E+300+1E+300i",
    "9.99999999999999E+307+9.99999999999999E+307i",
    "-9.99999999999999E+307-1E-300j",
    "2.22507385850721E-308+2.22507385850721E-308i",
    "1E-300-1E+300i",
)
"""Texts in each of the grammar's forms, the axes and both sides of the branch cut at -π."""


def _z(args: Sequence[MathArg]) -> mpmath.mpc:
    (z,) = args
    assert isinstance(z, mpmath.mpc)
    return z


def _modulus(args: Sequence[MathArg]) -> mpmath.mpf:
    z = _z(args)
    return mpmath.hypot(z.real, z.imag)


def _argument_domain(args: Sequence[MathArg]) -> str | None:
    z = _z(args)
    return "IMARGUMENT of 0 is #DIV/0!" if z.real == 0 and z.imag == 0 else None


def _argument(args: Sequence[MathArg]) -> mpmath.mpf:
    z = _z(args)
    if z.real != 0 and abs(z.imag / z.real) > DOUBLE_MAX:
        msg = "the quotient y/x overflows: ATAN2's #NUM! is unprobed for IMARGUMENT"
        raise NoValueError(msg)
    return mpmath.atan2(z.imag, z.real)


def _signed(rng: random.Random, lo: float, hi: float) -> float:
    x = round15(log_uniform(rng, lo, hi))
    return -x if rng.random() < 0.5 else x


def _text(rng: random.Random, re: float, im: float) -> str:
    return complex_text(round15(re), round15(im), rng.choice(COMPLEX_SUFFIXES))


def draw(stratum: str, rng: random.Random) -> list[Arg]:
    """One complex text: centre parts in [-100, 100]; the tail near an axis (one part 1E-15 to 1E-1 of the other,
    either sign, so the argument nears 0, ±π/2 and both sides of ±π); the edge texts, zero for a quarter of them; extreme parts of any magnitude;
    log-uniform parts over 1E-100 to 1E+100."""
    if stratum == "tail":
        big = rng.uniform(-100, 100) or 1.0
        small = big * _signed(rng, 1e-15, 1e-1)
        return [_text(rng, big, small) if rng.random() < 0.5 else _text(rng, small, big)]
    if stratum == "edge":
        return [rng.choice(ZERO_TEXTS if rng.random() < 0.25 else EDGE_TEXTS)]
    if stratum == "extreme":
        return [_text(rng, _signed(rng, MIN_LITERAL, MAX_LITERAL), _signed(rng, MIN_LITERAL, MAX_LITERAL))]
    if stratum == "log-uniform":
        return [_text(rng, _signed(rng, 1e-100, 1e100), _signed(rng, 1e-100, 1e100))]
    return [_text(rng, rng.uniform(-100, 100), rng.uniform(-100, 100))]


def _param(source: str) -> Param:
    return Param(
        "inumber", "text", typical=(-100.0, 100.0), extreme=(MIN_LITERAL, MAX_LITERAL), source=source
    )


IMABS = register(
    Definition(
        function="IMABS",
        params=(_param(KERNEL_SOURCE),),
        value=_modulus,
        draw=draw,
        sources=(KERNEL_SOURCE, GRAMMAR_SOURCE),
    )
)
IMARGUMENT = register(
    Definition(
        function="IMARGUMENT",
        params=(_param(KERNEL_SOURCE),),
        value=_argument,
        domain=_argument_domain,
        draw=draw,
        sources=(KERNEL_SOURCE, ZERO_SOURCE, GRAMMAR_SOURCE),
    )
)
