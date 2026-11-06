"""`literal.decimal15`: a double's exact decimal value cut to 15 significant digits (T-3745)."""

from __future__ import annotations

from decimal import ROUND_DOWN, ROUND_HALF_EVEN, Decimal

from rustsheet_tools.accuracy import literal


def test_default_is_half_even() -> None:
    assert literal.decimal15(0.1) == Decimal("0.100000000000000")
    assert literal.decimal15(2.675) == literal.decimal15(2.675, ROUND_HALF_EVEN)


def test_rounding_modes_differ_on_the_dropped_digits() -> None:
    x = 1 / 3 * 2  # 0.6666666666666666: the 16th digit rounds up, truncation keeps 6
    assert literal.decimal15(x) == Decimal("0.666666666666667")
    assert literal.decimal15(x, ROUND_DOWN) == Decimal("0.666666666666666")


def test_zero_and_carry() -> None:
    assert literal.decimal15(0.0, ROUND_DOWN) == 0
    assert literal.decimal15(9.9999999999999999e22) == Decimal("1.00000000000000E+23")
