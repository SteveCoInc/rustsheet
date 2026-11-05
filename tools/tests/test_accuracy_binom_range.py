"""Gate A definitions of BINOM.DIST.RANGE, BINOM.INV and CRITBINOM (T-3741) against published values and the
incomplete beta."""

from __future__ import annotations

import random

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import binom_range, discrete
from rustsheet_tools.accuracy.model import REGISTRY, DomainError
from rustsheet_tools.accuracy.sampler import STRATA

mp = mpmath.mpf


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(function: str, *args: float) -> mpmath.mpf:
    return REGISTRY[function].evaluate([mp(a) for a in args])


def test_the_summed_tails_agree_with_the_incomplete_beta() -> None:
    rng = random.Random(3741)
    for _ in range(60):
        n = mp(rng.randint(1, 300))
        k = mp(rng.randint(0, int(n)))
        p = mp(rng.uniform(0.01, 0.99))
        eps = mp(10) ** -35
        assert mpmath.almosteq(binom_range.binom_cdf_summed(k, n, p), discrete.binom_cdf(k, n, p), eps)
        assert mpmath.almosteq(binom_range.binom_sf_summed(k, n, p), discrete.binom_sf(k, n, p), eps)


def test_binom_dist_range_matches_the_published_example_and_the_pmf() -> None:
    # Microsoft's example: BINOM.DIST.RANGE(60,0.75,48) = 0.084 and (60,0.75,45,50) = 0.524
    assert mpmath.almosteq(value("BINOM.DIST.RANGE", 60, 0.75, 48), mp("0.0839749674"), mp(10) ** -9)
    assert mpmath.almosteq(value("BINOM.DIST.RANGE", 60, 0.75, 45, 50), mp("0.5236297934"), mp(10) ** -9)
    total = mpmath.fsum(discrete.binom_pmf(mp(k), mp(40), mp(0.3)) for k in range(5, 31))
    assert mpmath.almosteq(value("BINOM.DIST.RANGE", 40, 0.3, 5, 30), total, mp(10) ** -35)
    upper = mpmath.fsum(discrete.binom_pmf(mp(k), mp(40), mp(0.3)) for k in range(25, 34))
    assert mpmath.almosteq(value("BINOM.DIST.RANGE", 40, 0.3, 25, 33), upper, mp(10) ** -35)
    assert value("BINOM.DIST.RANGE", 2, 1, 0) == 0  # statistics.md: 0, not BINOM.DIST's #NUM! (T-2435)
    assert value("BINOM.DIST.RANGE", 0, 0.4, 0) == 1


def test_binom_dist_range_domain_follows_excel() -> None:
    for args in [(10, 0.5, 11), (10, 0.5, 3, 2), (10, 0.5, 3, 11), (2**31 - 1, 0.5, 0), (10, 1.5, 3)]:
        with pytest.raises(DomainError):
            value("BINOM.DIST.RANGE", *args)


def test_binom_dist_range_near_the_mean_at_the_trial_limit() -> None:
    # mpmath's 2F1 does not converge here; the summed tails do, and two complementary tails sum to 1
    n, p = mp(2**31 - 2), mp(0.3)
    k = mpmath.floor(n * p)
    assert mpmath.almosteq(
        binom_range.binom_cdf_summed(k, n, p) + binom_range.binom_sf_summed(k, n, p), 1, mp(10) ** -35
    )


@pytest.mark.parametrize("function", ["BINOM.INV", "CRITBINOM"])
def test_binom_inv_is_the_smallest_count_reaching_alpha(function: str) -> None:
    assert value(function, 6, 0.5, 0.75) == 4  # Microsoft's example
    rng = random.Random(1)
    for _ in range(40):
        n, p = rng.randint(1, 200), rng.uniform(0.01, 0.99)
        alpha = rng.choice(
            (rng.uniform(0.001, 0.999), 10 ** -rng.uniform(1, 300), 1 - 10 ** -rng.uniform(1, 15))
        )
        k = value(function, n, p, alpha)
        assert discrete.binom_cdf(k, mp(n), mp(p)) >= mp(alpha)
        assert k == 0 or discrete.binom_cdf(k - 1, mp(n), mp(p)) < mp(alpha)
    for args in [(10, 0, 0.5), (10, 1, 0.5), (10, 0.5, 0), (10, 0.5, 1), (2**31 - 1, 0.5, 0.5)]:
        with pytest.raises(DomainError):
            value(function, *args)


def test_binom_inv_at_the_trial_limit_is_the_median() -> None:
    # statistics.md "BINOM.INV and CRITBINOM take BINOM.DIST's trial limit": 1073741823 at 2^31 - 2 trials
    assert value("BINOM.INV", 2**31 - 2, 0.5, 0.5) == 1073741823


@pytest.mark.parametrize("function", ["BINOM.DIST.RANGE", "BINOM.INV"])
def test_every_stratum_draws_a_point_of_the_right_shape(function: str) -> None:
    definition = REGISTRY[function]
    assert definition.draw is not None
    rng = random.Random(7)
    for stratum in STRATA:
        for _ in range(20):
            args = definition.draw(stratum, rng)
            definition.params_for(len(args))  # an argument count the definition takes
            assert all(isinstance(a, (int, float)) for a in args)
