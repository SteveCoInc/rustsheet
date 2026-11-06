"""CONVERT(number, from_unit, to_unit) (T-4085).

The math: the exact conversion of the written number's double, with each unit's documented factor and Excel's
documented constant roundings, at the working precision. The units are T-4084's fixed-list text kind: every unit
name and every prefixed name the semantics entries evidence.

Sources:
- engineering.md, "CONVERT unit names" (T-3817): the unit names, the units that take each of the 21 decimal prefixes
  (`da` and its alias `e` are deka), the binary prefixes on `bit` and `byte` only, the prefix squared on `m2`, `ang2`
  (and their `^2` spellings) and cubed on `m3`, `ang3` (and `^3`) but taken once on `ar`, and the reading of a string
  within the class of the other unit, a unit name beating a prefixed reading.
- engineering.md, "CONVERT constants and evaluation order" (T-0922, T-1297, T-3861, T-3975):
  - The value is multiplied by the from-factor and divided by the to-factor, then scaled by the prefixes. Each factor
    is in its class's internal base (force in dynes, energy in ergs, volume in litres, area in square angstroms).
  - A factor is the unit's exact definition below (`_FACTORS`), except where the entry or the T-0922 fit to
    ironcalc-calc CONVERT.xlsx documents Excel's own constant: `sg` 14593.902937206363 g, `parsec`
    3.0856775812815532E+16 m, `eV` 1.602176487E-12 erg (CODATA 2006), `u` 1.660538782E-24 g (CODATA 2006),
    `mmHg` 133.322 Pa. Other factors' doubles differ from their definitions by at most 4.5E-16 relative
    (the engine's fitted table, `fn_convert`), which the envelope absorbs.
  - Two decimal prefixes are one constant, the double nearest their combined power of ten. With a binary prefix on
    either unit, each unit's prefixes are its own constants: the from-prefix's double, times the double nearest the
    reciprocal of the to-prefix's double, times the binary prefixes' exact ratio.
  - The arithmetic between those constants is exact here: which double operation Excel rounds where is not part of
    the definition (the envelope judges it; T-2046, T-2047).
  - `CONVERT(1E290,"m2","m2")` is 1E+290 although the product `value * F_from` overflows; a result past the double
    range is `#NUM!` (the generator redraws: no value). Where `value * F_from`, or the quotient by the to-factor before
    the prefixes, would be a nonzero below the smallest normal double, or the quotient overflows, no probe shows what
    Excel does, so the point has no value (NoValueError).
- engineering.md, "CONVERT temperature" (T-1297, T-1860): each pair's own form, with its constants as doubles
  (273.15, 1.8, 459.67, 0.8; 1.25, 2.25 and 32 are exact). Only `K` and `kel` take a prefix: the value is scaled by
  the from-prefix first and the result divided by the to-prefix (`CONVERT(1E-24,"yK","K")`, ov12).
- engineering.md, "CONVERT: a boolean unit is #VALUE! (T-2676)", probes/eng-convert-boolean-unit A30 and
  probes/convert-temperature-ov10, ov11: units of two classes are `#N/A` (a domain error).
"""

from __future__ import annotations

import random
from collections.abc import Sequence
from fractions import Fraction

import mpmath

from rustsheet_tools.accuracy.literal import Arg, round15
from rustsheet_tools.accuracy.model import Definition, DomainError, MathArg, NoValueError, Param, register
from rustsheet_tools.accuracy.sampler import log_uniform
from rustsheet_tools.accuracy.tables import DOUBLE_MAX

NAMES_SOURCE = "engineering.md#convert-unit-names"
ORDER_SOURCE = "engineering.md#convert-constants-and-evaluation-order"
TEMPERATURE_SOURCE = "engineering.md#convert-temperature"
MISMATCH_SOURCE = (
    "engineering.md#convert-a-boolean-unit-is-value-t-2676 (A30), probes/convert-temperature-ov10, ov11"
)
SMALLEST_NORMAL = mpmath.mpf(2) ** -1022

_LBM = Fraction("453.59237")  # g (the international avoirdupois pound)
_FT = Fraction("0.3048")  # m
_IN = Fraction("0.0254")  # m
_MI = Fraction("1609.344")  # m
_NMI = Fraction(1852)  # m
_YD = Fraction("0.9144")  # m
_GN = Fraction("9.80665")  # m/s² (standard gravity)
_LY = Fraction(299792458) * Fraction("365.25") * 86400  # m (Julian year of light)
_PICA_POINT = _IN / 72  # m
_SURVEY_FT = Fraction(1200, 3937)  # m
_GAL = Fraction("3.785411784")  # l (US gallon)
_HP = 550 * _FT * _LBM * _GN / 1000  # W (550 ft·lbf/s)
_LBF = _LBM * _GN * 100  # dyn
_EXCEL_DOUBLE = Fraction  # a documented Excel constant, exact as its double (see the module doc)

# Each class's factors in Excel's internal base unit (engineering.md "CONVERT constants and evaluation order").
_FACTORS: dict[str, dict[str, Fraction]] = {
    "area": {  # square angstroms (m2 = 1E20)
        "uk_acre": Fraction("4046.8564224") * 10**20,
        "us_acre": 43560 * (_SURVEY_FT) ** 2 * 10**20,
        "ang2": Fraction(1),
        "ang^2": Fraction(1),
        "ar": Fraction(10**22),
        "ft2": _FT**2 * 10**20,
        "ft^2": _FT**2 * 10**20,
        "ha": Fraction(10**24),
        "in2": _IN**2 * 10**20,
        "in^2": _IN**2 * 10**20,
        "ly2": _LY**2 * 10**20,
        "ly^2": _LY**2 * 10**20,
        "m2": Fraction(10**20),
        "m^2": Fraction(10**20),
        "Morgen": Fraction(2500 * 10**20),
        "mi2": _MI**2 * 10**20,
        "mi^2": _MI**2 * 10**20,
        "Nmi2": _NMI**2 * 10**20,
        "Nmi^2": _NMI**2 * 10**20,
        "Picapt2": _PICA_POINT**2 * 10**20,
        "Pica2": _PICA_POINT**2 * 10**20,
        "Pica^2": _PICA_POINT**2 * 10**20,
        "Picapt^2": _PICA_POINT**2 * 10**20,
        "yd2": _YD**2 * 10**20,
        "yd^2": _YD**2 * 10**20,
    },
    "distance": {  # metres
        "m": Fraction(1),
        "mi": _MI,
        "Nmi": _NMI,
        "in": _IN,
        "ft": _FT,
        "yd": _YD,
        "ang": Fraction(1, 10**10),
        "ell": 45 * _IN,
        "ly": _LY,
        "parsec": _EXCEL_DOUBLE(3.0856775812815532e16),
        "pc": _EXCEL_DOUBLE(3.0856775812815532e16),
        "Picapt": _PICA_POINT,
        "Pica": _PICA_POINT,
        "pica": _IN / 6,
        "survey_mi": 5280 * _SURVEY_FT,
    },
    "energy": {  # ergs (J = 1E7)
        "J": Fraction(10**7),
        "e": Fraction(1),
        "c": Fraction("4.184") * 10**7,
        "cal": Fraction("4.1868") * 10**7,
        "eV": _EXCEL_DOUBLE(1.602176487e-12),
        "ev": _EXCEL_DOUBLE(1.602176487e-12),
        "HPh": _HP * 3600 * 10**7,
        "hh": _HP * 3600 * 10**7,
        "Wh": Fraction(3600 * 10**7),
        "wh": Fraction(3600 * 10**7),
        "flb": _FT * _LBF * 100,  # ft·lbf: m·dyn is 100 erg
        "BTU": Fraction("1055.05585262") * 10**7,
        "btu": Fraction("1055.05585262") * 10**7,
    },
    "force": {  # dynes (N = 1E5)
        "N": Fraction(10**5),
        "dyn": Fraction(1),
        "dy": Fraction(1),
        "lbf": _LBF,
        "pond": _GN * 100,
    },
    "information": {"bit": Fraction(1), "byte": Fraction(8)},
    "magnetism": {"T": Fraction(1), "ga": Fraction(1, 10**4)},
    "power": {"HP": _HP, "h": _HP, "PS": 75 * _GN, "W": Fraction(1), "w": Fraction(1)},  # watts
    "pressure": {  # pascals
        "Pa": Fraction(1),
        "p": Fraction(1),
        "atm": Fraction(101325),
        "at": Fraction(101325),
        "mmHg": Fraction("133.322"),
        "psi": _LBF / 10**5 / _IN**2,
        "Torr": Fraction(101325, 760),
    },
    "speed": {  # metres per second
        "admkn": 6080 * _FT / 3600,
        "kn": _NMI / 3600,
        "m/h": Fraction(1, 3600),
        "m/hr": Fraction(1, 3600),
        "m/s": Fraction(1),
        "m/sec": Fraction(1),
        "mph": _MI / 3600,
    },
    "temperature": {name: Fraction(1) for name in ("C", "cel", "F", "fah", "K", "kel", "Rank", "Reau")},
    "time": {  # seconds
        "yr": Fraction("365.25") * 86400,
        "day": Fraction(86400),
        "d": Fraction(86400),
        "hr": Fraction(3600),
        "mn": Fraction(60),
        "min": Fraction(60),
        "sec": Fraction(1),
        "s": Fraction(1),
    },
    "volume": {  # litres (m3 = 1000)
        "tsp": _GAL / 768,
        "tspm": Fraction(5, 1000),
        "tbs": _GAL / 256,
        "oz": _GAL / 128,
        "cup": _GAL / 16,
        "pt": _GAL / 8,
        "us_pt": _GAL / 8,
        "uk_pt": Fraction("0.56826125"),
        "qt": _GAL / 4,
        "uk_qt": Fraction("1.1365225"),
        "gal": _GAL,
        "uk_gal": Fraction("4.54609"),
        "l": Fraction(1),
        "L": Fraction(1),
        "lt": Fraction(1),
        "ang3": Fraction(1, 10**27),
        "ang^3": Fraction(1, 10**27),
        "barrel": 42 * _GAL,
        "bushel": Fraction("35.23907016688"),
        "ft3": _FT**3 * 1000,
        "ft^3": _FT**3 * 1000,
        "in3": _IN**3 * 1000,
        "in^3": _IN**3 * 1000,
        "ly3": _LY**3 * 1000,
        "ly^3": _LY**3 * 1000,
        "m3": Fraction(1000),
        "m^3": Fraction(1000),
        "mi3": _MI**3 * 1000,
        "mi^3": _MI**3 * 1000,
        "yd3": _YD**3 * 1000,
        "yd^3": _YD**3 * 1000,
        "Nmi3": _NMI**3 * 1000,
        "Nmi^3": _NMI**3 * 1000,
        "Picapt3": _PICA_POINT**3 * 1000,
        "Picapt^3": _PICA_POINT**3 * 1000,
        "Pica3": _PICA_POINT**3 * 1000,
        "Pica^3": _PICA_POINT**3 * 1000,
        "GRT": 100 * _FT**3 * 1000,
        "regton": 100 * _FT**3 * 1000,
        "MTON": 40 * _FT**3 * 1000,
    },
    "weight": {  # grams
        "g": Fraction(1),
        "sg": _EXCEL_DOUBLE(14593.902937206363),
        "lbm": _LBM,
        "u": _EXCEL_DOUBLE(1.660538782e-24),
        "ozm": _LBM / 16,
        "grain": _LBM / 7000,
        "cwt": 100 * _LBM,
        "shweight": 100 * _LBM,
        "uk_cwt": 112 * _LBM,
        "lcwt": 112 * _LBM,
        "stone": 14 * _LBM,
        "ton": 2000 * _LBM,
        "brton": 2240 * _LBM,
        "LTON": 2240 * _LBM,
        "uk_ton": 2240 * _LBM,
    },
}

DECIMAL_PREFIXES = {
    "Y": 24, "Z": 21, "E": 18, "P": 15, "T": 12, "G": 9, "M": 6, "k": 3, "h": 2, "da": 1, "e": 1,
    "d": -1, "c": -2, "m": -3, "u": -6, "n": -9, "p": -12, "f": -15, "a": -18, "z": -21, "y": -24,
}  # fmt: skip
BINARY_PREFIXES = {"ki": 10, "Mi": 20, "Gi": 30, "Ti": 40, "Pi": 50, "Ei": 60, "Zi": 70, "Yi": 80}
DECIMAL_PREFIXED = frozenset([
    "Pa", "p", "atm", "at", "mmHg", "g", "u", "m", "ang", "ly", "parsec", "pc", "ang2", "ang^2", "ar", "m2", "m^2",
    "N", "dyn", "dy", "pond", "J", "e", "c", "cal", "eV", "ev", "Wh", "wh", "W", "w", "T", "ga", "uk_pt", "l", "L",
    "lt", "ang3", "ang^3", "m3", "m^3", "bit", "byte", "m/h", "m/hr", "m/s", "m/sec", "mph", "K", "kel", "sec", "s",
])  # fmt: skip
BINARY_PREFIXED = frozenset({"bit", "byte"})
_SQUARED = frozenset({"m2", "m^2", "ang2", "ang^2"})  # `ar` is an area that takes its prefix once
_CUBED = frozenset({"m3", "m^3", "ang3", "ang^3"})


def _power(name: str) -> int:
    return 2 if name in _SQUARED else 3 if name in _CUBED else 1


def _choices() -> tuple[str, ...]:
    out: dict[str, None] = {}
    for units in _FACTORS.values():
        for name in units:
            out[name] = None
            if name in DECIMAL_PREFIXED:
                for prefix in DECIMAL_PREFIXES:
                    out[prefix + name] = None
            if name in BINARY_PREFIXED:
                for prefix in BINARY_PREFIXES:
                    out[prefix + name] = None
    return tuple(out)


UNITS = _choices()
"""Every unit string the entries evidence: each unit name, and each prefix on each unit that takes it."""


def read_unit(units: dict[str, Fraction], text: str) -> tuple[str, Fraction, int, int] | None:
    """`text` read within one class: (unit name, factor, decimal exponent of its prefix, already times 2 or 3 for a
    squared or cubed unit, binary exponent of its prefix); None when it does not read there. A unit name wins, then
    the longest unit name a prefix can precede (engineering.md "CONVERT unit names")."""
    if text in units:
        return text, units[text], 0, 0
    for name in sorted(units, key=lambda n: (-len(n), n)):
        prefix = text.removesuffix(name)
        if prefix == text or not prefix:
            continue
        if name in DECIMAL_PREFIXED and prefix in DECIMAL_PREFIXES:
            return name, units[name], DECIMAL_PREFIXES[prefix] * _power(name), 0
        if name in BINARY_PREFIXED and prefix in BINARY_PREFIXES:
            return name, units[name], 0, BINARY_PREFIXES[prefix]
    return None


def class_of(
    from_unit: str, to_unit: str
) -> tuple[str, tuple[str, Fraction, int, int], tuple[str, Fraction, int, int]]:
    """The first class (by name) in which both strings read, with both readings; DomainError (`#N/A`) when none."""
    for kind in sorted(_FACTORS):
        a, b = read_unit(_FACTORS[kind], from_unit), read_unit(_FACTORS[kind], to_unit)
        if a is not None and b is not None:
            return kind, a, b
    msg = f"{from_unit!r} and {to_unit!r} share no class (#N/A)"
    raise DomainError(msg)


def _double(x: float) -> mpmath.mpf:
    return mpmath.mpf(x)


def _power_of_ten(n: int) -> mpmath.mpf:
    """The double nearest 10^n (Python's float parsing is correctly rounded)."""
    return _double(float(f"1e{n}"))


# A temperature pair's form with its constants as doubles (engineering.md "CONVERT temperature").
_TEMPERATURE_NAMES = {
    "C": "C",
    "cel": "C",
    "F": "F",
    "fah": "F",
    "K": "K",
    "kel": "K",
    "Rank": "Rank",
    "Reau": "Reau",
}
_C0 = _double(273.15)
_R = _double(1.8)
_RF = _double(459.67)
_RE = _double(0.8)


def _temperature(v: mpmath.mpf, source: str, target: str) -> mpmath.mpf:
    pair = (source, target)
    if source == target:
        return v
    forms = {
        ("C", "F"): lambda: v * _R + 32,
        ("C", "K"): lambda: v + _C0,
        ("C", "Rank"): lambda: (v + _C0) * _R,
        ("C", "Reau"): lambda: v * _RE,
        ("F", "C"): lambda: (v - 32) / _R,
        ("F", "K"): lambda: (v - 32) / _R + _C0,
        ("F", "Rank"): lambda: v + _RF,
        ("F", "Reau"): lambda: (v - 32) * _RE / _R,
        ("K", "C"): lambda: v - _C0,
        ("K", "F"): lambda: (v - _C0) * _R + 32,
        ("K", "Rank"): lambda: v * _R,
        ("K", "Reau"): lambda: (v - _C0) * _RE,
        ("Rank", "C"): lambda: v / _R - _C0,
        ("Rank", "F"): lambda: v - _RF,
        ("Rank", "K"): lambda: v / _R,
        ("Rank", "Reau"): lambda: (v / _R - _C0) * _RE,
        ("Reau", "C"): lambda: v * mpmath.mpf(1.25),
        ("Reau", "F"): lambda: v * mpmath.mpf(2.25) + 32,
        ("Reau", "K"): lambda: v * mpmath.mpf(1.25) + _C0,
        ("Reau", "Rank"): lambda: (v * mpmath.mpf(1.25) + _C0) * _R,
    }
    return forms[pair]()


def _unprobed_tiny(x: mpmath.mpf, what: str) -> None:
    if x != 0 and abs(x) < SMALLEST_NORMAL:
        msg = f"{what} is below the smallest normal double: unprobed"
        raise NoValueError(msg)


def _parts(args: Sequence[MathArg]) -> tuple[mpmath.mpf, str, str]:
    v, from_unit, to_unit = args
    assert isinstance(from_unit, str) and isinstance(to_unit, str)
    assert isinstance(v, mpmath.mpf)
    return v, from_unit, to_unit


def _domain(args: Sequence[MathArg]) -> str | None:
    _, from_unit, to_unit = _parts(args)
    try:
        class_of(from_unit, to_unit)
    except DomainError as err:
        return str(err)
    return None


def _convert(args: Sequence[MathArg]) -> mpmath.mpf:
    v, from_unit, to_unit = _parts(args)
    kind, (source, f_from, dec_from, bin_from), (target, f_to, dec_to, bin_to) = class_of(from_unit, to_unit)
    if kind == "temperature":
        scaled = v * _power_of_ten(dec_from)
        _unprobed_tiny(scaled, "the prefixed value")
        return _temperature(scaled, _TEMPERATURE_NAMES[source], _TEMPERATURE_NAMES[target]) / _power_of_ten(
            dec_to
        )
    product = v * mpmath.mpf(f_from.numerator) / f_from.denominator
    _unprobed_tiny(product, "value * from-factor")
    scaled = product * f_to.denominator / f_to.numerator
    _unprobed_tiny(scaled, "the conversion before the prefixes")
    if abs(scaled) > DOUBLE_MAX:
        msg = "the conversion before the prefixes overflows: unprobed"
        raise NoValueError(msg)
    if bin_from or bin_to:
        reciprocal = _double(1 / float(_power_of_ten(dec_to)))
        prefixes = _power_of_ten(dec_from) * reciprocal * mpmath.mpf(2) ** (bin_from - bin_to)
    else:
        prefixes = _power_of_ten(dec_from - dec_to)
    return scaled * prefixes


# -- the draw ------------------------------------------------------------------------------------------------------


def _unit(rng: random.Random, kind: str) -> str:
    """A unit string of class `kind`: a unit name, or (half the time, where the class has one) a prefixed name that
    reads back as that unit in the class."""
    names = list(_FACTORS[kind])
    prefixable = [n for n in names if n in DECIMAL_PREFIXED]
    while True:
        if not prefixable or rng.random() < 0.5:
            return rng.choice(names)
        name = rng.choice(prefixable)
        if name in BINARY_PREFIXED and rng.random() < 0.3:
            text = rng.choice(list(BINARY_PREFIXES)) + name
        else:
            text = rng.choice(list(DECIMAL_PREFIXES)) + name
        reading = read_unit(_FACTORS[kind], text)
        if reading is not None and reading[0] == name:  # not shadowed by a unit name or a longer one
            return text


def _signed(rng: random.Random, lo: float, hi: float) -> float:
    x = log_uniform(rng, lo, hi)
    return -x if rng.random() < 0.5 else x


EDGE_VALUES = (0.0, 1.0, -1.0, 1e300, -1e300, 1e-300, 32.0, 273.15, -273.15, -459.67, 459.67, 100.0)


def draw(stratum: str, rng: random.Random) -> list[Arg]:
    """One CONVERT call: a class, two of its unit strings (the edge stratum also pairs two classes, `#N/A`), and a
    value: [-1000, 1000] typical, 1E-15 to 1E-1 or 1E+5 to 1E+15 in the tail, the edge values (temperature zeros,
    ±1E+300 as probes/convert-temperature-ov*), 1E-150 to 1E+150 extreme and 1E-250 to 1E+250 log-uniform (the range
    of probes-numeric t3975-convert-mixed-prefix-sweep). Values of either sign (the T-3861 and T-3975 sweeps)."""
    kinds = sorted(_FACTORS)
    kind = rng.choice(kinds)
    from_unit, to_unit = _unit(rng, kind), _unit(rng, kind)
    if stratum == "tail":
        value = _signed(rng, 1e-15, 1e-1) if rng.random() < 0.5 else _signed(rng, 1e5, 1e15)
    elif stratum == "edge":
        value = rng.choice(EDGE_VALUES)
        if rng.random() < 0.25:
            to_unit = _unit(rng, rng.choice([k for k in kinds if k != kind]))
    elif stratum == "extreme":
        value = _signed(rng, 1e-150, 1e150)
    elif stratum == "log-uniform":
        value = _signed(rng, 1e-250, 1e250)
    else:
        value = rng.uniform(-1000, 1000)
    return [round15(value), from_unit, to_unit]


CONVERT = register(
    Definition(
        function="CONVERT",
        params=(
            Param("number", "real", typical=(-1000.0, 1000.0), extreme=(1e-150, 1e150), source=ORDER_SOURCE),
            Param("from_unit", "text", choices=UNITS, source=NAMES_SOURCE),
            Param("to_unit", "text", choices=UNITS, source=NAMES_SOURCE),
        ),
        value=_convert,
        domain=_domain,
        draw=draw,
        sources=(NAMES_SOURCE, ORDER_SOURCE, TEMPERATURE_SOURCE, MISMATCH_SOURCE),
    )
)
