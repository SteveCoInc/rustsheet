"""TIME, HOUR, MINUTE and SECOND (Gate A slice 4k, T-3678, decision 0228).

Each reads Excel's rule from dates.md, VERIFIED on excel-graph cells:
- TIME (dates.md "TIME: an argument past 32767 is `#NUM!`; a near-integer hour counts as that
  integer" and "TIME divides by 86400 first, then drops whole days"): each argument is floored on
  its own, except that a value within 2^-22 + 2^-33 below an integer counts as that integer (half
  that window for a negative value; the engine's `near_integer_whole`, whose constants lie inside
  the measured brackets). An argument above 32767 after that read is `#NUM!`; one below -32768
  wraps by 65536 into -32768..32767 (probes/t3678-time-argument-far-negative, excel-graph,
  2026-11-03: `TIME(600,-32769,0)` is `TIME(600,32767,0)`, `TIME(10,0,-98304)` is
  `TIME(10,0,-32768)`; on 2026-10-21 the same Excel gave `#NUM!`, dates.md records the change).
  A negative total is `#NUM!`. The math is the exact `((h*3600 + m*60 + s)/86400) mod 1`; Excel divides in
  double precision first, which moves the last bits only (`TIME(32767,0,0)` is
  0.29166666666674246, 2.6E-13 relative from 7/24), far inside Gate A's envelope. The draws keep
  every argument's fraction outside the window's unprobed edge zone (between 2^-22 + 1E-12 and
  2.4E-7 below an integer, half that for a negative value).
- HOUR, MINUTE and SECOND (dates.md "HOUR, MINUTE and SECOND round to the nearest second", all 40
  cells of probes/second-rounding): the time of day rounds to the nearest second by Excel's own
  double steps, `y = x + 0.5/86400`, `h = (y - floor(y))*24`, `m = (h - floor(h))*60`, and the
  parts are `floor(h)`, `floor(m)` and `floor((m - floor(m))*60)`. An exact half goes whichever way
  the steps' binary noise takes it, so the rule is defined on the doubles and the math takes it
  from them (as MROUND's step does, decision 0246): an exact-second reading would put a tie a whole
  unit from Excel, a false decision 0188 R3 mark. A negative serial is `#NUM!`, and so is one whose
  carried day `floor(x + 0.5/86400)` is past 2958465 (dates.md "HOUR, MINUTE and SECOND past
  9999-12-31": `fl(2958466 - 0.5/86400)` and above are `#NUM!`).
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.defs._common import by_stratum, fraction_to_mpf, one_of, scalars
from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, Interval, MathArg, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform

TIME_BOUND_SOURCE = (
    "dates.md#time-an-argument-past-32767-is-num-a-near-integer-hour-counts-as-that-integer-t-2267"
)
TIME_DIVIDE_SOURCE = "dates.md#time-divides-by-86400-first-then-drops-whole-days"
PARTS_SOURCE = "dates.md#hour-minute-and-second-round-to-the-nearest-second"
PARTS_BOUND_SOURCE = "dates.md#hour-minute-and-second-past-9999-12-31"
DECISION = "decision 0228"

NEAR_INTEGER_TOLERANCE = 2.0**-22 + 2.0**-33  # the engine's `NEAR_INTEGER_TOLERANCE`
ARGUMENT_MIN, ARGUMENT_MAX = -32768, 32767  # TIME's argument range, after the near-integer read
WRAP = 65536  # an argument below -32768 wraps by this into the range
SECONDS_PER_DAY = 86400
LAST_DAY = 2958465  # 9999-12-31
HALF_SECOND = 0.5 / SECONDS_PER_DAY  # the double Excel adds before it splits the time of day


def time_argument(x: float) -> int:
    """One TIME argument as Excel reads it: floored, but a value within the near-integer window
    below an integer counts as that integer (half the window for a negative value). A whole value
    above 32767 is `#NUM!` (DomainError); one below -32768 wraps by 65536 into -32768..32767
    (probes/t3678-time-argument-far-negative, probed down to -131072)."""
    tolerance = NEAR_INTEGER_TOLERANCE / 2 if x < 0 else NEAR_INTEGER_TOLERANCE
    whole = math.floor(x)
    if x - whole >= 1 - tolerance:  # exact for every finite double
        whole += 1
    if whole > ARGUMENT_MAX:
        msg = f"a TIME argument reads as {whole}, above 32767"
        raise DomainError(msg)
    if whole < ARGUMENT_MIN:
        whole = (whole - ARGUMENT_MIN) % WRAP + ARGUMENT_MIN
    return whole


def time_value(hour: float, minute: float, second: float) -> Fraction:
    """TIME's math: the exact fraction of a day of the read arguments, whole days dropped.
    DomainError for an argument out of bounds or a negative total."""
    total = time_argument(hour) * 3600 + time_argument(minute) * 60 + time_argument(second)
    if total < 0:
        msg = "TIME's total is negative"
        raise DomainError(msg)
    day = Fraction(total, SECONDS_PER_DAY)
    return day - math.floor(day)


def clock_parts(serial: float) -> tuple[int, int, int]:
    """(HOUR, MINUTE, SECOND) of a serial by Excel's double steps (module docstring). DomainError
    (`#NUM!`) for a negative serial or one whose carried day is past 9999-12-31."""
    if serial < 0:
        msg = "a negative serial"
        raise DomainError(msg)
    y = serial + HALF_SECOND
    if math.floor(y) > LAST_DAY:
        msg = "the serial rounds past 9999-12-31"
        raise DomainError(msg)
    hours = (y - math.floor(y)) * 24
    hour = math.floor(hours)
    minutes = (hours - hour) * 60
    minute = math.floor(minutes)
    return hour, minute, math.floor((minutes - minute) * 60)


def _time(args: Sequence[MathArg]) -> mpmath.mpf:
    hour, minute, second = scalars(args)
    return fraction_to_mpf(time_value(float(hour), float(minute), float(second)))


def _part(index: int) -> Callable[[Sequence[MathArg]], mpmath.mpf]:
    def value(args: Sequence[MathArg]) -> mpmath.mpf:
        (serial,) = scalars(args)
        return mpmath.mpf(clock_parts(float(serial))[index])

    return value


def _domain(value: Callable[[Sequence[MathArg]], mpmath.mpf]) -> Callable[[Sequence[MathArg]], str | None]:
    def domain(args: Sequence[MathArg]) -> str | None:
        try:
            value(args)
        except DomainError as why:
            return str(why)
        return None

    return domain


# -- TIME draws --------------------------------------------------------------------------------


def _near_whole(rng: random.Random, whole: int) -> float:
    """`whole` less an offset that snaps (inside the window) or floors (well outside it); never in
    the window's unprobed edge zone. The offset is cut to the literal's 15 digits."""
    window = NEAR_INTEGER_TOLERANCE / 2 if whole <= 0 else NEAR_INTEGER_TOLERANCE
    if rng.random() < 0.5:
        offset = window * rng.choice((1e-8, 1e-4, 0.01, 0.1, 0.5))
    else:
        offset = rng.choice((1e-6, 1e-5, 1e-3, 0.25, 0.5, 0.75))
    return round15(whole - offset)


def _clock_args(rng: random.Random) -> list[Arg]:
    """A time of day: hour 0-23, minute and second 0-59, a second with a fraction one time in three."""
    second: float = rng.randrange(60)
    if rng.random() < 1 / 3:
        second = round15(second + rng.random())
    return [float(rng.randrange(24)), float(rng.randrange(60)), second]


def _time_tail(rng: random.Random) -> list[Arg]:
    """One argument just below an integer, inside or outside the near-integer window."""
    args = _clock_args(rng)
    i = rng.randrange(3)
    whole = rng.choice((1, 2, 5, 24, 60, 100, 1000, 30000, 32767, 32768, 0, -1, -5, -30000, -32767))
    args[i] = _near_whole(rng, whole)
    if whole <= 0:  # keep the total from going negative most of the time: the window is the point
        args[0] = float(rng.randint(1, 23)) if i != 0 else args[0]
    return args


def _time_edge(rng: random.Random) -> list[Arg]:
    """The argument bound (32767, 32767.5, 32768, -32768, -32769), totals of whole days, a total just
    below zero, and zeros."""
    bound = rng.choice((32767.0, 32767.5, 32767.9999, 32768.0, 65535.0, -32768.0, -32768.5, -32769.0))
    shapes: list[Callable[[], list[Arg]]] = [
        lambda: [bound, 0.0, 0.0],
        lambda: [600.0, bound, 0.0],
        lambda: [10.0, 0.0, bound],
        lambda: [32767.0, 32767.0, 32767.0],
        lambda: [float(24 * rng.randint(0, 1365)), 0.0, 0.0],  # whole days: 0
        lambda: [0.0, float(1440 * rng.randint(1, 22)), 0.0],
        lambda: [0.0, 0.0, -1.0],  # a negative total: #NUM!
        lambda: [1.0, -61.0, 0.0],
        lambda: [0.0, 0.0, -0.0005],
        lambda: [0.0, 0.0, 0.0],
        lambda: [23.0, 59.0, 59.0],
        lambda: [0.0, 0.5, 30.0],
    ]
    return one_of(rng, shapes)


def _time_extreme(rng: random.Random) -> list[Arg]:
    """Arguments over the whole -32768..32767 range (about one in five out of it), with fractions."""

    def arg() -> float:
        x = rng.uniform(-36000, 36000) if rng.random() < 0.2 else rng.uniform(-20000, 32767)
        return round15(x) if rng.random() < 0.5 else float(math.floor(x))

    hour, minute, second = arg(), arg(), arg()
    args: list[Arg] = [abs(hour), minute, second]  # a positive hour keeps most totals non-negative
    return args


def _time_log_uniform(rng: random.Random) -> list[Arg]:
    """Each argument log-uniform in [1E-6, 1E5] with a sign one time in five."""

    def arg() -> float:
        x = log_uniform(rng, 1e-6, 1e5)
        return -x if rng.random() < 0.2 else x

    return [arg(), arg(), arg()]


TIME = register(
    Definition(
        function="TIME",
        params=(
            Param("hour", "real", Interval(), source=TIME_BOUND_SOURCE),
            Param("minute", "real", Interval(), source=TIME_BOUND_SOURCE),
            Param("second", "real", Interval(), source=TIME_BOUND_SOURCE),
        ),
        value=_time,
        domain=_domain(_time),
        draw=by_stratum(
            {
                "typical": _clock_args,
                "tail": _time_tail,
                "edge": _time_edge,
                "extreme": _time_extreme,
                "log-uniform": _time_log_uniform,
            },
            "TIME draws: clock times, arguments beside an integer, the argument bound and whole days.",
        ),
        sources=(DECISION, TIME_BOUND_SOURCE, TIME_DIVIDE_SOURCE),
    )
)


# -- HOUR, MINUTE and SECOND draws ---------------------------------------------------------------


def _serial_typical(rng: random.Random) -> list[Arg]:
    """A date from 1900 to about 2064 with a time of day, as a 15-digit literal."""
    return [round15(rng.randrange(0, 60000) + rng.random())]


def _serial_tail(rng: random.Random) -> list[Arg]:
    """A time near a half second (where the rounding decides), at an exact second, or within a
    second of midnight (the carry into the next day)."""
    day = rng.choice((0, 1, 59, 60, 45000, rng.randrange(1, 60000), LAST_DAY - 1))
    shape = rng.randrange(3)
    if shape == 0:
        seconds = (
            rng.randrange(SECONDS_PER_DAY) + 0.5 + rng.choice((0.0, 1e-3, -1e-3, 0.01, -0.01, 0.1, -0.1))
        )
    elif shape == 1:
        seconds = float(rng.randrange(SECONDS_PER_DAY))
    else:
        seconds = SECONDS_PER_DAY - rng.choice((0.4, 0.49, 0.51, 0.6, 1.0, 1e-3))
    return [round15(day + seconds / SECONDS_PER_DAY)]


def _serial_edge(rng: random.Random) -> list[Arg]:
    """Zero, a negative serial (`#NUM!`), and serials at the last date's half-second cut."""
    return [
        rng.choice(
            (
                0.0,
                1e-10,
                -1e-10,
                -1.0,
                0.5,
                1.0,
                60.5,
                2958465.0,
                2958465.9,
                2958465.99999,
                2958465.9999942,
                2958465.9999949,
                2958466.0,
                2958466.5,
            )
        )
    ]


def _serial_extreme(rng: random.Random) -> list[Arg]:
    """Times late on 9999-12-31, and serials far past it (`#NUM!`) or far below a day."""
    shape = rng.randrange(3)
    if shape == 0:
        return [round15(LAST_DAY + rng.uniform(0.9, 0.99999))]
    if shape == 1:
        return [round15(log_uniform(rng, 3e6, 1e15))]
    return [round15(log_uniform(rng, 1e-12, 1e-4))]


def _serial_log_uniform(rng: random.Random) -> list[Arg]:
    """A serial log-uniform in [1E-6, 2958465]."""
    return [round15(log_uniform(rng, 1e-6, float(LAST_DAY)))]


_PARTS_DRAW = by_stratum(
    {
        "typical": _serial_typical,
        "tail": _serial_tail,
        "edge": _serial_edge,
        "extreme": _serial_extreme,
        "log-uniform": _serial_log_uniform,
    },
    "HOUR, MINUTE and SECOND draws: times of day, half seconds, midnight and the last date's cut.",
)


def _part_definition(function: str, index: int) -> Definition:
    value = _part(index)
    return register(
        Definition(
            function=function,
            params=(Param("serial_number", "real", Interval(), source=PARTS_SOURCE),),
            value=value,
            domain=_domain(value),
            draw=_PARTS_DRAW,
            sources=(DECISION, PARTS_SOURCE, PARTS_BOUND_SOURCE),
        )
    )


HOUR = _part_definition("HOUR", 0)
MINUTE = _part_definition("MINUTE", 1)
SECOND = _part_definition("SECOND", 2)
