"""Gate A definitions of IMABS, IMARGUMENT and CONVERT (T-4085)."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.complex_modulus import EDGE_TEXTS, IMABS, IMARGUMENT, ZERO_TEXTS
from rustsheet_tools.accuracy.defs.convert import CONVERT, UNITS, class_of, read_unit
from rustsheet_tools.accuracy.literal import round15
from rustsheet_tools.accuracy.model import NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg
from rustsheet_tools.accuracy.tables import compute
from rustsheet_tools.fitlab.sweep import parse_complex_text


def close(value: str | None, excel: float, rel: float = 4.5e-16) -> bool:
    """`value` (the math) is within `rel` of Excel's double."""
    assert value is not None
    with mpmath.workdps(60):  # independent of the precision another test leaves behind
        m = mpmath.mpf(value)
        return bool(abs(m - mpmath.mpf(excel)) <= rel * abs(m))


def rounds_to(value: str | None, excel: float) -> bool:
    """`value` (the math) rounds to Excel's double."""
    assert value is not None
    return float(mpmath.mpf(value)) == excel


def test_imabs_and_imargument_values() -> None:
    assert compute(IMABS, ["3+4i"]).value == "5.0"
    assert compute(IMABS, ["-3-4j"]).value == "5.0"
    assert compute(IMABS, ["0"]).value == "0.0"
    with mpmath.workdps(40):
        assert mpmath.mpf(compute(IMARGUMENT, ["-1+0i"]).value) == +mpmath.pi
        assert mpmath.mpf(compute(IMARGUMENT, ["-j"]).value) == -mpmath.pi / 2
    # engineering.md "IM* reciprocal kernels": Excel's 956.6150000078482 is 2 ulps above the math (B33)
    assert close(compute(IMABS, ["956.615+0.00387493i"]).value, 956.6150000078482, 1e-15)
    # engineering.md T-2420 entry: `IMARGUMENT("1.5E+2-2E-3i")` is -1.3333333332543209E-05
    assert close(compute(IMARGUMENT, ["1.5E+2-2E-3i"]).value, -1.3333333332543209e-05)
    # the branch cut: a negative real part with a tiny imaginary part of either sign
    assert mpmath.mpf(compute(IMARGUMENT, ["-1+1E-300i"]).value) > 3.14
    assert mpmath.mpf(compute(IMARGUMENT, ["-1-1E-300j"]).value) < -3.14


def test_imargument_of_zero_is_div0_and_an_overflowing_quotient_has_no_value() -> None:
    assert compute(IMARGUMENT, ["0"]).why == "IMARGUMENT of 0 is #DIV/0!"
    assert compute(IMARGUMENT, ["0+0i"]).why == "IMARGUMENT of 0 is #DIV/0!"
    with pytest.raises(NoValueError):
        compute(IMARGUMENT, ["1E-300+1E+300i"])
    assert compute(IMARGUMENT, ["0+1E+300i"]).value is not None  # x = 0 is ±π/2, no quotient


@pytest.mark.parametrize("definition", [IMABS, IMARGUMENT])
def test_the_complex_draws_are_readable_15_digit_texts(definition: object) -> None:
    sampler = Sampler(definition, random.Random(5))  # type: ignore[arg-type]
    for stratum in STRATA:
        for index in range(60):
            (text,) = sampler.draw(stratum, index)
            assert isinstance(text, str)
            re, im = parse_complex_text(text)
            for part in (re, im):
                assert part == 0 or 2.2250738585072014e-308 <= abs(part) < 1e308
                assert round15(part) == part
    for text in EDGE_TEXTS + ZERO_TEXTS:
        parse_complex_text(text)


def test_convert_documented_values() -> None:
    # engineering.md "CONVERT constants and evaluation order"
    assert compute(CONVERT, [5.0, "cal", "J"]).value == "20.934"
    assert close(compute(CONVERT, [5.0, "eV", "J"]).value, 8.010882434999999e-19)
    assert close(compute(CONVERT, [0.0002, "Mang3", "m3"]).value, 2.0000000000000002e-16)
    assert rounds_to(compute(CONVERT, [1.0, "sg", "g"]).value, 14593.902937206363)
    assert rounds_to(compute(CONVERT, [1.0, "parsec", "m"]).value, 3.0856775812815532e16)
    # the exact lbf (453.59237 g x 9.80665 m/s2), whose double is Excel's 444822.16152605
    assert close(compute(CONVERT, [1.0, "lbf", "dyn"]).value, 444822.16152605, 1.2e-16)
    assert close(compute(CONVERT, [5.0, "cal", "eV"]).value, 1.3065976295281882e20)
    assert close(compute(CONVERT, [1.0, "uk_pt", "gal"]).value, 0.15011874068810688)
    assert close(compute(CONVERT, [1.0, "tsp", "tbs"]).value, 0.3333333333333333)
    assert close(
        compute(CONVERT, [1.0, "flb", "J"]).value, 1.3558179483314003
    )  # probes/convert-unit-names A481
    # prefixes: one constant, the double nearest the combined power of ten
    assert compute(CONVERT, [1.0, "m", "nm"]).value == "1000000000.0"
    assert rounds_to(compute(CONVERT, [1.0, "Pm3", "l"]).value, 9.999999999999999e47)
    assert close(compute(CONVERT, [1e300, "Ym", "Zm"]).value, 1e303)
    assert close(compute(CONVERT, [1.0, "kar", "m2"]).value, 1e5)  # `ar` takes its prefix once
    assert close(compute(CONVERT, [1.0, "km2", "m2"]).value, 1e6)
    # a binary prefix: each unit's own constant, the target's decimal prefix as its rounded reciprocal (T-3975)
    assert close(compute(CONVERT, [4.73e-10, "Gibyte", "zbyte"]).value, 5.0787988275200007e20)
    assert rounds_to(compute(CONVERT, [1.0, "kibyte", "bit"]).value, 8192.0)
    # temperature (engineering.md "CONVERT temperature")
    assert close(compute(CONVERT, [1.0, "F", "K"]).value, 255.92777777777775)
    assert close(compute(CONVERT, [0.1, "K", "Rank"]).value, 0.18000000000000002)
    assert close(compute(CONVERT, [300.0, "mK", "K"]).value, 0.3)
    assert close(compute(CONVERT, [1e-24, "yK", "K"]).value, 9.999999999999998e-49)
    assert compute(CONVERT, [273.15, "K", "C"]).value == "0.0"  # the constants are doubles: no residue


def test_convert_reads_a_string_within_the_other_unit_s_class() -> None:
    assert close(compute(CONVERT, [1.0, "pc", "c"]).value, 1e-12)  # pico-calories against `c`
    assert rounds_to(
        compute(CONVERT, [1.0, "pc", "m"]).value, 3.0856775812815532e16
    )  # the parsec against `m`
    assert class_of("Pica", "m")[1][0] == "Pica"  # a unit name beats a prefixed reading
    assert close(compute(CONVERT, [1.0, "mmmHg", "Pa"]).value, 0.133322)
    assert close(compute(CONVERT, [1.0, "kmph", "m/s"]).value, 447.04)
    assert read_unit({"in": mpmath.mpf(1)}, "kin") is None  # type: ignore[dict-item]


def test_convert_units_of_two_classes_are_na_and_overflow_has_no_value() -> None:
    assert compute(CONVERT, [1.0, "Ym", "K"]).why == "'Ym' and 'K' share no class (#N/A)"
    assert compute(CONVERT, [1.0, "m", "s"]).why is not None
    with pytest.raises(NoValueError):
        compute(CONVERT, [1e300, "Ym", "m"])  # Excel's #NUM! (ov02): the math is beyond the double range
    with pytest.raises(NoValueError):
        compute(CONVERT, [2.3e-308, "u", "g"])  # value * from-factor underflows: unprobed


def test_the_unit_list_holds_the_evidenced_names_only() -> None:
    assert len(UNITS) == len(set(UNITS))
    for name in ("m", "kg", "Gibyte", "kibit", "mK", "dam", "eatm", "km^2", "yang3", "survey_mi", "regton"):
        assert name in UNITS
    for name in ("kmi", "kft", "mC", "kF", "kim", "Mig", "kiK", "KM"):
        assert name not in UNITS


def test_convert_draws_one_class_and_the_edge_also_pairs_two() -> None:
    sampler = Sampler(CONVERT, random.Random(9))
    mismatched = 0
    for stratum in STRATA:
        for index in range(80):
            value, from_unit, to_unit = sampler.draw(stratum, index)
            assert isinstance(value, float) and value == round15(value)
            assert from_unit in UNITS and to_unit in UNITS
            args = [math_arg(p, a) for p, a in zip(CONVERT.params, (value, from_unit, to_unit), strict=True)]
            why = CONVERT.domain(args) if CONVERT.domain else None
            if why is not None:
                assert stratum == "edge"
                mismatched += 1
    assert mismatched > 0
