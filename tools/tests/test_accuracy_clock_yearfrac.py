"""Gate A definitions of TIME, HOUR, MINUTE, SECOND and YEARFRAC (T-3678) against dates.md's
excel-graph values."""

from __future__ import annotations

from fractions import Fraction

import pytest

from rustsheet_tools.accuracy.defs.clock import clock_parts, time_value
from rustsheet_tools.accuracy.defs.yearfrac import yearfrac_value
from rustsheet_tools.accuracy.model import DomainError

DAY = 86400


@pytest.mark.parametrize(
    ("args", "seconds"),
    [
        ((24, 2, 3), 123),  # TIME(24,2,3): whole days dropped
        ((32767, 0, 0), 32767 * 3600 % DAY),
        ((600, -32768, 0), (600 * 3600 - 32768 * 60) % DAY),
        ((10, 0, -32768), (36000 - 32768) % DAY),
        ((1 - 2**-22, 0, 0), 3600),  # inside the window: 1:00
        ((1 - 2.4e-7, 0, 0), 0),  # outside it: 0:00
        ((10, 0, -1e-7), 36000),  # a negative inside its half window
        ((10, 0, -1.2e-7), 35999),
        ((1, -0.5, 0), 3540),
        ((0, 0.5, 30), 30),
        ((23.9999999, 0, 0), 0),
        ((32767.9999, 0, 0), 32767 * 3600 % DAY),
        # probes/t3678-time-argument-far-negative: below -32768 an argument wraps by 65536.
        ((600, -32769, 0), (600 * 3600 + 32767 * 60) % DAY),
        ((600, -32768.5, 0), (600 * 3600 + 32767 * 60) % DAY),
        ((-32769, 0, 0), 32767 * 3600 % DAY),
        ((10, 0, -98304), 36000 - 32768),
        ((14120, -131072, 0), 14120 * 3600 % DAY),
        ((14120, -33313.544720951, -1389.80591442708), 59930),
    ],
)
def test_time_reads_each_argument(args: tuple[float, float, float], seconds: int) -> None:
    assert time_value(*args) == Fraction(seconds, DAY)


@pytest.mark.parametrize(
    "args",
    [
        (32768, 0, 0),
        (0, 32769, 0),
        (0, 0, 65536),
        (-32768, 0, 0),
        (-65537, 0, 0),
        (-0.5, 0, 1),
        (0, 0, -0.0005),
    ],
)
def test_time_num(args: tuple[float, float, float]) -> None:
    with pytest.raises(DomainError):
        time_value(*args)


def _time(h: int, m: int, s: int) -> float:
    return (h * 3600 + m * 60 + s) / DAY


@pytest.mark.parametrize(
    ("serial", "parts"),
    [
        (_time(10, 59, 59) + 0.6 / DAY, (11, 0, 0)),
        (0.5 + 1.5 / DAY, (12, 0, 1)),  # probes/second-rounding: ties go by the steps' noise
        (0.5 + 2.5 / DAY, (12, 0, 2)),
        (0.5 + 3.5 / DAY, (12, 0, 4)),
        (1 + 1.5 / DAY, (0, 0, 2)),
        (45000 + 1.5 / DAY, (0, 0, 1)),
        (0.5 + 59.5 / DAY, (12, 0, 59)),
        (2958465.9, (21, 36, 0)),
        (2958465.99999, (23, 59, 59)),
        (2958465.9999942123, (23, 59, 59)),
    ],
)
def test_clock_parts(serial: float, parts: tuple[int, int, int]) -> None:
    assert clock_parts(serial) == parts


@pytest.mark.parametrize("serial", [-1e-10, 2958466.0, 2958465.9999942128, 2958465.9999949, 123456789.123])
def test_clock_parts_num(serial: float) -> None:
    with pytest.raises(DomainError):
        clock_parts(serial)


@pytest.mark.parametrize(
    ("start", "end", "basis", "value"),
    [
        (60, 59, 0, Fraction(-1, 360)),
        (60, 61, 0, Fraction(2, 360)),
        (0, 59, 0, Fraction(58, 360)),
        (0, 60, 0, Fraction(59, 360)),
        (0, 61, 0, Fraction(61, 360)),
        (0, 366, 0, Fraction(361, 360)),
        (60, 425, 0, Fraction(359, 360)),
        (59, 425, 0, Fraction(1)),
        (55862, 60, 0, Fraction(55000, 360)),
        (1, 30145, 0, Fraction(29712, 360)),
        (0, 60, 3, Fraction(60, 365)),
        (59, 60, 2, Fraction(1, 360)),
        (59, 60, 1, Fraction(1, 365)),
        (60, 61, 1, Fraction(1, 365)),
        (1, 366, 1, Fraction(1)),
        (1, 367, 1, Fraction(366, 365)),
        (1, 60, 1, Fraction(59, 365)),
        (1, 366, 2, Fraction(365, 360)),
        (60, 366, 3, Fraction(306, 365)),
        (1, 366, 4, Fraction(359, 360)),
        (60, 366, 4, Fraction(301, 360)),
        (0, 31, 4, Fraction(30, 360)),
        (59, 60, 4, Fraction(1, 360)),
        (60, 61, 4, Fraction(2, 360)),
        (0.5, 0.25, 2, Fraction(0)),
    ],
)
def test_yearfrac(start: float, end: float, basis: int, value: Fraction) -> None:
    assert yearfrac_value(start, end, basis) == value


def test_yearfrac_swaps_and_averages() -> None:
    # dates.md "YEARFRAC swaps reversed dates": 2024-03-31 back to 2024-01-15 on basis 0 is 76/360.
    assert yearfrac_value(45382, 45306, 0) == Fraction(76, 360)
    # "YEARFRAC accepts serial 0": serial 0 is a day of 1900 on basis 1.
    assert float(yearfrac_value(0, 33752, 1)) == pytest.approx(92.4086198775318, rel=1e-14)
    assert float(yearfrac_value(1, 33752, 1)) == pytest.approx(92.4058820065944, rel=1e-14)
    assert float(yearfrac_value(48344, 1, 1)) == pytest.approx(132.35660175388034, rel=1e-14)
    assert float(yearfrac_value(60, 1520, 1)) == pytest.approx(3.9978094194961664, rel=1e-14)


@pytest.mark.parametrize(
    ("start", "end", "basis"), [(-1, 10, 0), (10, -1e-10, 1), (1, 2958466, 2), (1, 2, 5)]
)
def test_yearfrac_num(start: float, end: float, basis: int) -> None:
    with pytest.raises(DomainError):
        yearfrac_value(start, end, basis)
