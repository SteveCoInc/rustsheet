"""Gate A definitions of HYPGEOM.DIST and HYPGEOMDIST, and the legacy BINOMDIST, NEGBINOMDIST and POISSON (T-4105),
against Microsoft's published examples and the semantics entries' Excel values."""

from __future__ import annotations

import random
from fractions import Fraction

import mpmath
import pytest

from rustsheet_tools.accuracy.defs import discrete
from rustsheet_tools.accuracy.defs._common import fraction_to_mpf
from rustsheet_tools.accuracy.model import REGISTRY, DomainError
from rustsheet_tools.accuracy.sampler import STRATA, math_arg

mp = mpmath.mpf
LIMIT = 2**31 - 1  # the first count Excel rejects (statistics.md T-2255)


@pytest.fixture(autouse=True)
def precision() -> object:
    with mpmath.workdps(40):
        yield


def value(function: str, *args: float | bool) -> mpmath.mpf:
    return REGISTRY[function].evaluate([a if isinstance(a, bool) else mp(a) for a in args])


def exactly(got: mpmath.mpf, num: int, den: int) -> bool:
    return bool(mpmath.almosteq(got, fraction_to_mpf(Fraction(num, den)), mp(10) ** -35))


def test_hypgeom_matches_the_published_example() -> None:
    # Microsoft's example: HYPGEOM.DIST(1,4,8,20,TRUE) 0.4654 and FALSE 0.3633, C(8,1) C(12,3) / C(20,4) = 352/969
    assert exactly(value("HYPGEOM.DIST", 1, 4, 8, 20, False), 352, 969)
    assert exactly(value("HYPGEOM.DIST", 1, 4, 8, 20, True), 451, 969)
    assert exactly(value("HYPGEOMDIST", 1, 4, 8, 20), 352, 969)


def test_hypgeom_is_the_exact_rational_excel_misses_by_a_few_ulp() -> None:
    # statistics.md "HYPGEOM.DIST is not exact for small integer arguments": Excel 0.5000000000000001 and
    # 0.15909090909090923, the exact values 1/2 and 7/44
    assert exactly(value("HYPGEOMDIST", 0, 3, 1, 6), 1, 2)
    assert exactly(value("HYPGEOM.DIST", 0, 3, 5, 12, False), 7, 44)
    assert exactly(value("HYPGEOM.DIST", 1, 4, 8, 22, True), 559, 1045)


def test_hypgeom_outside_the_support_is_zero_or_one() -> None:
    # statistics.md T-1048 (`HYPGEOMDIST(5,4,8,20)` 0) and T-2255 (`HYPGEOMDIST(3,5,5,6)` 0, below the support)
    assert value("HYPGEOMDIST", 5, 4, 8, 20) == 0
    assert value("HYPGEOM.DIST", 5, 4, 8, 20, True) == 1
    assert value("HYPGEOMDIST", 3, 5, 5, 6) == 0
    assert value("HYPGEOM.DIST", 3, 5, 5, 6, True) == 0
    assert value("HYPGEOMDIST", 1, 0, 8, 20) == 0
    assert value("HYPGEOMDIST", 0, 0, 8, 20) == 1


def test_hypgeom_domain_follows_excel() -> None:
    # statistics.md T-1048: population_s or number_sample above number_pop; T-2255: a count of 2^31 - 1 or a negative one
    for args in [
        (1, 4, 25, 20),
        (1, 4, 8, 5),
        (1, 25, 8, 20),
        (1, 4, 8, 0),
        (-1, 4, 8, 20),
        (1, 2, 3, LIMIT),
    ]:
        with pytest.raises(DomainError):
            value("HYPGEOMDIST", *args)
        with pytest.raises(DomainError):
            value("HYPGEOM.DIST", *args, True)
    assert value("HYPGEOMDIST", 1, 2, 3, LIMIT - 1) > 0  # 2147483646 computes (T-2255)
    with pytest.raises(DomainError):
        value("HYPGEOMDIST", LIMIT, 2, 2, 17)  # even above the support


def test_hypgeom_cdf_is_the_sum_of_its_pmfs_on_both_sides_of_the_mode() -> None:
    n, k, pop = mp(40), mp(70), mp(200)  # support 0..40, mode 14
    for x in (0, 3, 13, 14, 25, 39):
        total = mpmath.fsum(discrete.hypgeom_pmf(mp(j), n, k, pop) for j in range(x + 1))
        assert mpmath.almosteq(discrete.hypgeom_cdf(mp(x), n, k, pop), total, mp(10) ** -35)


def test_hypgeom_deep_tail_keeps_its_digits() -> None:
    n, k, pop = mp(500), mp(5000), mp(10000)
    x = mp(150)  # about 9 standard deviations below the mean
    total = mpmath.fsum(discrete.hypgeom_pmf(mp(j), n, k, pop) for j in range(151))
    got = discrete.hypgeom_cdf(x, n, k, pop)
    assert got < mp(10) ** -19
    assert mpmath.almosteq(got, total, mp(10) ** -30)


@pytest.mark.parametrize("function", ["HYPGEOM.DIST", "HYPGEOMDIST"])
def test_every_stratum_draws_a_point_the_math_evaluates(function: str) -> None:
    definition = REGISTRY[function]
    assert definition.draw is not None
    rng = random.Random(7)
    for stratum in STRATA:
        for _ in range(20):
            args = definition.draw(stratum, rng)
            params = definition.params_for(len(args))
            assert len(args) == (5 if function == "HYPGEOM.DIST" else 4)
            try:
                result = definition.evaluate([math_arg(p, a) for p, a in zip(params, args, strict=True)])
            except DomainError:
                continue
            assert 0 <= result <= 1


def test_a_wide_cdf_is_drawn_as_a_pmf() -> None:
    definition = REGISTRY["HYPGEOM.DIST"]
    assert definition.draw is not None
    rng = random.Random(11)
    for _ in range(200):
        _, n, k, pop, cumulative = definition.draw("extreme", rng)
        assert isinstance(n, int) and isinstance(k, int) and isinstance(pop, int)
        if cumulative:
            share = k / pop
            assert (n * share * (1 - share) * (pop - n) / (pop - 1)) ** 0.5 <= discrete.HYPGEOM_SD_CAP


def test_legacy_names_reuse_the_modern_definitions() -> None:
    # statistics.md "Legacy distribution names sweep (T-1047)": `BINOMDIST(2.9,10,0.5,TRUE)` = `BINOMDIST(2,...)`
    assert exactly(value("BINOMDIST", 2, 10, 0.5, True), 56, 1024)
    assert value("BINOMDIST", 6, 10, 0.5, False) == value("BINOM.DIST", 6, 10, 0.5, False)
    with pytest.raises(DomainError):
        value("BINOMDIST", 0, 3, 1, False)  # the p = 1 pmf at k = 0 (T-2435)
    with pytest.raises(DomainError):
        value("BINOMDIST", 0, LIMIT, 0.5, True)  # the trial limit (T-2645)
    # NEGBINOMDIST is NEGBINOM.DIST's pmf (T-1048); Microsoft's example NEGBINOMDIST(10,5,0.25) = 0.0550487
    assert value("NEGBINOMDIST", 10, 5, 0.25) == value("NEGBINOM.DIST", 10, 5, 0.25, False)
    assert mpmath.almosteq(value("NEGBINOMDIST", 10, 5, 0.25), mp("0.0550487"), mp(10) ** -7)
    with pytest.raises(DomainError):
        value("NEGBINOMDIST", 0, 3, 1)  # `NEGBINOMDIST(0,3,1)` is #NUM! (T-1048)
    # POISSON is POISSON.DIST, the negative-mean quirk included (`POISSON(0,-1,TRUE)` = e)
    assert value("POISSON", 0, -1, True) == mpmath.e
    assert value("POISSON", 2, 5, False) == value("POISSON.DIST", 2, 5, False)
    with pytest.raises(DomainError):
        value("POISSON", 1, -1, True)
