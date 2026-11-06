"""Gate A definitions of GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, BETA.DIST and BETA.INV (T-3529) against published values."""

from __future__ import annotations

from collections.abc import Iterator

import mpmath
import pytest

from rustsheet_tools.accuracy import tables
from rustsheet_tools.accuracy.defs import beta, gamma
from rustsheet_tools.accuracy.model import REGISTRY, DomainError, NoValueError

mp = mpmath.mpf


def close(value: mpmath.mpf, published: str, digits: int) -> bool:
    # within 10^-digits, relative above 1 and absolute below
    return bool(mpmath.almosteq(value, mp(published), mp(10) ** -digits, mp(10) ** -digits))


@pytest.fixture(autouse=True)
def precision() -> Iterator[None]:
    with mpmath.workdps(50):
        yield


def test_gamma_and_chi_squared_quantiles_match_the_published_examples() -> None:
    # Microsoft's examples, to the digits shown: GAMMA.INV(0.068094,9,2) = 10.0000112,
    # CHISQ.INV(0.93,1) = 5.318520074, CHISQ.INV(0.6,2) = 1.832581464, CHISQ.INV.RT(0.050001,10) = 18.30697346.
    assert close(REGISTRY["GAMMA.INV"].evaluate([mp("0.068094"), mp(9), mp(2)]), "10.0000112", 6)
    assert close(REGISTRY["CHISQ.INV"].evaluate([mp("0.6"), mp(2)]), "1.832581464", 9)
    assert close(REGISTRY["CHISQ.INV.RT"].evaluate([mp("0.050001"), mp(10)]), "18.30697346", 8)
    # CHISQ.INV at df 2 is the exponential quantile -2 ln(1 - p) (statistics.md, T-2860).
    p = mp("0.3")
    assert close(REGISTRY["CHISQ.INV"].evaluate([p, mp(2)]), str(-2 * mpmath.log1p(-p)), 40)
    # df truncates: 2.9 is 2.
    assert REGISTRY["CHISQ.INV"].evaluate([p, mp("2.9")]) == REGISTRY["CHISQ.INV"].evaluate([p, mp(2)])


def test_incomplete_gamma_tails_agree_with_mpmath_where_it_converges() -> None:
    for a, y in [(mp("2.5"), mp(3)), (mp(40), mp(25)), (mp(40), mp(70)), (mp("0.3"), mp("1e-5"))]:
        assert close(gamma.lower_tail(y, a), str(mpmath.gammainc(a, 0, y, regularized=True)), 45)
        assert close(gamma.upper_tail(y, a), str(mpmath.gammainc(a, y, mpmath.inf, regularized=True)), 45)


def test_gamma_quantile_at_a_large_shape_round_trips() -> None:
    # mpmath's gammainc does not converge here; the series and continued fraction do.
    a, p = mp(100000), mp("1e-50")
    y = gamma.quantile(p, a, upper=False)
    assert close(gamma.lower_tail(y, a) / p, "1", 30)
    y = gamma.quantile(p, a, upper=True)
    assert close(gamma.upper_tail(y, a) / p, "1", 30)


def test_temme_coefficients_match_the_published_constants() -> None:
    # c_0(0) = -1/3, c_0'(0) = 1/12, c_1(0) = -1/540 and c_2(0) = 25/6048 (Temme 1979; DLMF 8.12.11): the recursion's
    # constants are found by cancelling the pole, so these check them.
    c = gamma.temme_coefficients(90)
    assert close(c[0][0], str(mp(-1) / 3), 45)
    assert close(c[0][1], str(mp(1) / 12), 45)
    assert close(c[1][0], str(mp(-1) / 540), 45)
    assert close(c[2][0], str(mp(25) / 6048), 45)


def test_temme_expansion_agrees_with_the_series_and_mpmath() -> None:
    # Where the summed series and continued fraction run (to TEMME_SHAPE_MIN), the expansion matches them to 40 digits,
    # from z = -37 to 37 standard deviations (tails from about 1E-300); at 1E4 mpmath's gammainc agrees to 48.
    for a in (mp(10000), mp(1000000)):
        for z in (-37, -5, 0, mp("0.5"), 3, 37):
            y = a + z * mpmath.sqrt(a)
            lower, upper = gamma._temme(y, a)
            assert close(
                lower / gamma._series(y, a) if y < a + 1 else lower / (1 - gamma._continued_fraction(y, a)),
                "1",
                40,
            )
            assert close(
                upper / (1 - gamma._series(y, a)) if y < a + 1 else upper / gamma._continued_fraction(y, a),
                "1",
                40,
            )
    a = mp(10000)
    for y in (a - 20 * mpmath.sqrt(a), a + 3 * mpmath.sqrt(a)):
        with mpmath.workdps(70):
            reference = mpmath.gammainc(a, 0, y, regularized=True)
        assert close(gamma._temme(y, a)[0] / reference, "1", 48)


def test_gamma_quantiles_at_a_huge_shape_are_excels_true_quantile() -> None:
    # Where Excel's search converges it is the true quantile (statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"):
    # GAMMA.INV(0.5,5E9,1) = 4999999999.666667, GAMMA.INV(1E-30,7E9,1) = 6999040894.354145 and
    # CHISQ.INV(1E-100,6E9) = 5997669910.951994, now from Temme's expansion.
    assert close(
        REGISTRY["GAMMA.INV"].evaluate([mp("0.5"), mp("5e9"), mp(1)]) / mp("4999999999.666667"), "1", 15
    )
    assert close(
        REGISTRY["GAMMA.INV"].evaluate([mp("1e-30"), mp("7e9"), mp(1)]) / mp("6999040894.354145"), "1", 15
    )
    assert close(REGISTRY["CHISQ.INV"].evaluate([mp("1e-100"), mp("6e9")]) / mp("5997669910.951994"), "1", 15)
    # At 1E300 the tails keep their sign: the exponent is summed with log10(a) more digits.
    a = mp("1e300")
    assert gamma.lower_tail(a * mp("0.6"), a) > 0
    assert gamma.upper_tail(a * mp("1.5"), a) > 0
    # The quantile is a + z sqrt(a) + ...: at 50 digits it is a itself, a relative 1E-149 apart.
    y = gamma.quantile(mp("1e-300"), a, upper=True)
    assert close(y / a, "1", 45)


def test_temme_tails_have_no_value_far_from_the_shape() -> None:
    # Past |eta| = 0.5 the coefficient series are not summed; the quantile's bracket never goes there.
    for tail in (gamma.lower_tail, gamma.upper_tail):
        with pytest.raises(NoValueError):
            tail(mp("1e9"), mp("1e8"))


def test_beta_matches_the_published_examples() -> None:
    # Microsoft's BETA.DIST(2,8,10,TRUE,1,3) = 0.6854706 and BETA.INV(0.685470581,8,10,1,3) = 2: on [0, 1] that is
    # x = 0.5; the density BETA.DIST(2,8,10,FALSE,1,3) = 1.4837646 is the [0, 1] density at 0.5 over B - A = 2.
    assert close(REGISTRY["BETA.DIST"].evaluate([mp("0.5"), mp(8), mp(10), True]), "0.6854706", 7)
    assert close(REGISTRY["BETA.DIST"].evaluate([mp("0.5"), mp(8), mp(10), False]) / 2, "1.4837646", 7)
    assert close(REGISTRY["BETA.INV"].evaluate([mp("0.685470581"), mp(8), mp(10)]), "0.5", 9)
    # the upper-tail branch: I_x(2, 1) = x^2
    assert close(beta.quantile(mp("0.81"), mp(2), mp(1)), "0.9", 40)


def test_beta_density_is_a_domain_error_at_an_end_with_a_shape_up_to_1() -> None:
    dist = REGISTRY["BETA.DIST"]
    with pytest.raises(DomainError):
        dist.evaluate([mp(0), mp(1), mp(2), False])
    with pytest.raises(DomainError):
        dist.evaluate([mp(1), mp(2), mp("0.5"), False])
    assert dist.evaluate([mp(0), mp("1.5"), mp(2), False]) == 0
    assert dist.evaluate([mp(0), mp("0.5"), mp(2), True]) == 0


def test_beta_quantile_has_no_value_where_the_root_is_below_the_smallest_double() -> None:
    # T-3578: BETAINV is #NUM! where its quantile underflows (statistics.md, T-2435), so the log bracket stops at
    # 2^-1074. probes-numeric/t1543-inverse-cdf-tiny-p A91 BETA.INV(1E-300,0.5,100) has math 7.87E-603.
    with pytest.raises(NoValueError):
        REGISTRY["BETA.INV"].evaluate([mp("1E-300"), mp("0.5"), mp(100)])
    # A root just inside the double range keeps its value: I_x(1, 1) = x.
    assert close(beta.quantile(mp("1E-320"), mp(1), mp(1)), "1E-320", 330)


def test_gamma_quantile_keeps_a_value_below_the_smallest_double() -> None:
    # Excel gives 0 there and committed s1 points rely on it (gamma.quantile's docstring); the value is the math.
    value = REGISTRY["CHISQ.INV"].evaluate([mp("1E-300"), mp(1)])
    assert 0 < value < mp("1E-500")


def test_beta_cdf_past_mpmaths_hypsum_takes_the_positive_series() -> None:
    # probes-numeric/t3571-r5-bfrac-fresh A239: mpmath's betainc gives up at these shapes (T-3680).
    a, b, x = mp(4563.6), mp(2816.5), mp(0.583019617141068)  # the cell's doubles
    with pytest.raises(ValueError, match="hypsum"):
        mpmath.betainc(a, b, 0, x, regularized=True)
    assert close(beta.lower_tail(x, a, b), "3.0490854915703403050837858706446013617166928e-10", 40)


def test_beta_tail_series_agrees_with_mpmath_where_it_converges() -> None:
    # the last two are past the switch point (a + 1) / (a + b + 2): the other side's complement
    for a, b, x in (
        (mp("2.5"), mp(7), mp("0.1")),
        (mp(40), mp("3.5"), mp("0.95")),
        (mp(200), mp(150), mp("0.6")),
    ):
        assert mpmath.almosteq(
            beta._series(a, b, x), mpmath.betainc(a, b, 0, x, regularized=True), mp(10) ** -45
        )


def test_beta_inv_past_the_switch_point_at_a_skewed_shape_stays_in_budget() -> None:
    # probes-numeric/t3443-basym-sweep A150: the classic side choice summed ~6E4 terms per call and the
    # root solve ran out of its work budget; main's mpmath value is 0.0061942141340931975... (T-3680).
    value = tables.compute(REGISTRY["BETA.INV"], [0.665410001077291, 383.0, 62750.652])
    assert value.value.startswith("0.0061942141340931975056697909694474336368428487")
