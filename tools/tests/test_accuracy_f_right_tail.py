"""Gate A definitions of F.DIST.RT, F.INV.RT, FDIST and FINV (T-4103) against published and semantics values."""

from __future__ import annotations

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import fisher
from rustsheet_tools.accuracy.model import REGISTRY, DomainError

mp = mpmath.mpf


def close(value: mpmath.mpf, published: str, digits: int) -> bool:
    # within 10^-digits, relative above 1 and absolute below
    return bool(mpmath.almosteq(value, mp(published), mp(10) ** -digits, mp(10) ** -digits))


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


@pytest.mark.parametrize("function", ["F.DIST.RT", "FDIST"])
def test_right_tail_matches_the_published_example(function: str) -> None:
    # Microsoft's F.DIST.RT and FDIST example: (15.2068649, 6, 4) = 0.01.
    assert close(REGISTRY[function].evaluate([mp("15.2068649"), mp(6), mp(4)]), "0.01", 8)


@pytest.mark.parametrize("function", ["F.INV.RT", "FINV"])
def test_right_tail_inverse_matches_the_published_example(function: str) -> None:
    # Microsoft's F.INV.RT and FINV example: (0.01, 6, 4) = 15.206865.
    assert close(REGISTRY[function].evaluate([mp("0.01"), mp(6), mp(4)]), "15.206865", 5)


def test_right_tail_values_from_the_semantics_entries() -> None:
    # Excel's values, within 15 to 16 digits of the math (statistics.md "Legacy distribution names sweep (T-1047)" and
    # "Thin distribution functions (T-1049)"): FDIST(1,2,5), FINV(0.5,2,5), F.INV.RT(0.5,3,7).
    assert close(REGISTRY["FDIST"].evaluate([mp(1), mp(2), mp(5)]), "0.43120115037169215", 16)
    assert close(REGISTRY["FINV"].evaluate([mp("0.5"), mp(2), mp(5)]), "0.7987697769322353", 15)
    assert close(REGISTRY["F.INV.RT"].evaluate([mp("0.5"), mp(3), mp(7)]), "0.8709442531872849", 15)
    # Excel's F.INV.RT(1E-16,9,3) = 60647458731.09812 (within 2E-15 of the math) needs the right tail solved
    # directly, not as F.INV(1 - p) ("F.INV is a crossing of F.DIST's tails").
    assert close(fisher.quantile(mp("1e-16"), mp(9), mp(3), upper=True) / mp("60647458731.09812"), "1", 13)


def test_the_right_tail_is_the_complement_and_the_inverse_round_trips() -> None:
    x, d1, d2 = mp("1.7"), mp(5), mp(11)
    rt = REGISTRY["F.DIST.RT"].evaluate([x, d1, d2])
    assert close(rt + fisher.lower_tail(x, d1, d2), "1", 35)
    assert close(REGISTRY["F.INV.RT"].evaluate([rt, d1, d2]), str(x), 30)
    assert REGISTRY["F.DIST.RT"].evaluate([mp(0), d1, d2]) == 1  # F.DIST.RT(0,3,7) = 1 (t1049-fdistrt A13)


@pytest.mark.parametrize(("modern", "legacy"), [("F.DIST.RT", "FDIST"), ("F.INV.RT", "FINV")])
def test_legacy_names_are_the_modern_definition_with_the_same_domain(modern: str, legacy: str) -> None:
    new, old = REGISTRY[modern], REGISTRY[legacy]
    assert [p.domain for p in new.params] == [p.domain for p in old.params]
    assert new.value is old.value and new.domain is old.domain
    args = [mp("0.3"), mp(4), mp(9)]
    assert new.evaluate(args) == old.evaluate(args)


@pytest.mark.parametrize("function", ["F.INV.RT", "FINV"])
def test_inverse_boundaries(function: str) -> None:
    # p = 1 is 0 and p = 0 is #NUM! on both names (`F.INV.RT(1,3,7)`, `FINV(1,2,5)`, `FINV(0,2,5)`).
    definition = REGISTRY[function]
    assert definition.evaluate([mp(1), mp(2), mp(5)]) == 0
    for bad in (
        [mp(0), mp(2), mp(5)],
        [mp("1.5"), mp(2), mp(5)],
        [mp("0.5"), mp(0), mp(5)],
        [mp("0.5"), mp(2), mp("1e11")],
    ):
        with pytest.raises(DomainError):
            definition.evaluate(bad)


@pytest.mark.parametrize("function", ["F.DIST.RT", "FDIST"])
def test_right_tail_errors(function: str) -> None:
    definition = REGISTRY[function]
    # x < 0 (`F.DIST.RT(-1,3,7)`, `FDIST(-1,2,5)`), df above 1E10 (T-1530), df1·x overflowing (`FDIST(1E308,2,3)`)
    # and df2/(df1·x) overflowing (`FDIST(1E-300,6,1E+10)`) are errors.
    for bad in (
        [mp(-1), mp(3), mp(7)],
        [mp(1), mp("1e11"), mp(7)],
        [mp("1e308"), mp(2), mp(3)],
        [mp("1e-300"), mp(6), mp("1e10")],
    ):
        with pytest.raises(DomainError):
            definition.evaluate(bad)
    # One power of ten either way keeps the quotient finite: `FDIST(1E-300,6,1E+9)` and `FDIST(1E-290,6,1E+10)` are 1;
    # x = 8E307 at df1 = 2 keeps df1·x = 1.6E308 finite: `F.DIST.RT(8E307,2,3)` and `FDIST(8E307,2,3)` are 0.
    assert close(definition.evaluate([mp("1e-300"), mp(6), mp("1e9")]), "1", 30)
    assert close(definition.evaluate([mp("1e-290"), mp(6), mp("1e10")]), "1", 30)
    assert close(definition.evaluate([mp("8e307"), mp(2), mp(3)]), "0", 30)
    # A subnormal x reads as 0 in Excel and never reaches the quotient: `FDIST(1E-308,6,1E+8)` is 1 (T-2435).
    assert close(definition.evaluate([mp("1e-308"), mp(6), mp("1e8")]), "1", 30)


def test_the_l140_bup_band_is_an_error_on_both_names() -> None:
    # `F.DIST.RT(1.5,79,1E10)` (t4017 ...-1e10 A209), `FDIST(2,79,5E9)` (t4045 E4) and `F.DIST.RT(2,77,1E10)` (t4030 A33)
    # are #NUM!; df1 76 and 80 at 1E10, 78 at 5E9 and 79 at 1E9 are numbers (T-4030, T-4044).
    assert fisher.bup_ratio_overflows(79, 1e10, 1.5)
    assert fisher.bup_ratio_overflows(79, 5e9, 2)
    assert fisher.bup_ratio_overflows(77, 1e10, 2)
    for df1, df2 in ((76, 1e10), (80, 1e10), (78, 5e9), (79, 1e9)):
        assert not fisher.bup_ratio_overflows(df1, df2, 2)
    assert not fisher.bup_ratio_overflows(79, 1e10, 0.5)  # below the mean bratio takes no L140
    with pytest.raises(DomainError):
        REGISTRY["FDIST"].evaluate([mp(2), mp(79), mp("5e9")])
    # L140 needs b0·x0 > 0.7 (statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s (T-3553)"): at x = 1E10,
    # (79, 5E9) has x0 = 1/159 and b0·x0 = 0.25, so bratio takes bpser and the band does not apply.
    assert not fisher.bup_ratio_overflows(79, 5e9, 1e10)
    domain = REGISTRY["FDIST"].domain
    assert domain is not None
    assert domain([mp("1e10"), mp(79), mp("5e9")]) is None
