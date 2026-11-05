"""T-4389 shared pieces: the T-4370 model (docs/research/t4370/model.py) and the two candidates this task scored.

Both were written down (2026-11-05) before probes/t4389-holdout was generated or scored. The constant is a fit
(FITTED): with an offset C added to T-4370's remainder before its 15-significant-digit rounding, every C in
[1.74E-17, 1.86E-17] s scores all 538 `.000` cells of the t4323/t4332/t4370 cases, and 2^-72 days (1.83E-17 s) is the
power of two inside that window. The open question was where Excel adds it:
- `seconds_digits`: C in seconds, added to the remainder, `.000` with r below 0.1 s;
- `days_digits`: 2^-72 added to the serial in days after the half step, before the cut. The addition rounds away once
  it is under half the serial's ulp (from 2^-18 days, about 0.33 s); in [2^-19, 2^-18) it is exactly half an ulp and
  rounds to even.
The hold-out's 31 cells after whole seconds separate them: the days variant scores all 368 cells and the seconds
variant misses those 31. The days variant was then frozen (ungated), and probes/t4389-fraction-fresh was drawn after
that. The review of #3440 asked for untouched cells on both the days-or-seconds choice and the band from 2^-19 days, so
probes/t4389-confirm (gen_confirm.py) was drawn after the review: days 40 of 40 after whole seconds (seconds 0), and
in the band the ungated model 29 of 40 against 11 for a 2^-19-day gate. The engine ships the ungated model.
"""
import math
import os
import sys
from decimal import ROUND_FLOOR, Decimal

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "../t4370"))
from model import chain_units, load, remainder, rnd  # noqa: E402,F401  (model.py adds ../t4332 to the path)
from model import fmt as _fmt  # noqa: E402

EXTRA = {"mm:ss.000": "{cm:02d}:{cs:02d}.{fr}", "s.000": "{cs}.{fr}", "[s].00": "{el}.{fr}"}


def fmt(f, s, fields):
    """model.py's text, plus the formats only probes/t4389-* use."""
    if f not in EXTRA:
        return _fmt(f, s, fields)
    frac, clock, el = fields
    fr = f"{frac:0{len(str(s)) - 1}d}"
    return EXTRA[f].format(cm=clock // 60 % 60, cs=clock % 60, el=el, fr=fr)

C_DAYS = 2.0 ** -72
C_SECONDS = C_DAYS * 86400
RANGE = 0.1


def r15(v):
    return Decimal(f"{v:.14e}") if v else Decimal(0)


def floor_digits(r, s):
    return int((r15(r) * s).to_integral_value(rounding=ROUND_FLOOR)) % s


def base_digits(x, s):
    """T-4370's digits."""
    return floor_digits(remainder(x, s), s)


def seconds_digits(x, s):
    """Candidate 1: C added to the remainder in seconds, `.000` and r < 0.1 s only."""
    r = remainder(x, s)
    return floor_digits(r + C_SECONDS if s == 1000 and r < RANGE else r, s)


def days_digits(x, s, gate=None):
    """Candidate 2: 2^-72 added to the serial after the half step, before the cut, `.000` only. With `gate`, only
    where the half-stepped serial is below it (the first version of the engine code gated at 2^-19 days)."""
    if s != 1000:
        return base_digits(x, s)
    v = x + 0.5 / (86400 * s)
    if gate is None or v < gate:
        v += C_DAYS
    v -= math.floor(v)
    for f in (24, 60, 60):
        v *= f
        v -= math.floor(v)
    return floor_digits(v, s)


def fields(digits_fn):
    def f(x, s):
        return digits_fn(x, s), chain_units(x, s) // s, rnd(x * 86400 * s) // s
    return f


def gated_digits(x, s):
    """The first engine version: candidate 2 gated at 2^-19 days (rejected by probes/t4389-confirm)."""
    return days_digits(x, s, 2.0 ** -19)
