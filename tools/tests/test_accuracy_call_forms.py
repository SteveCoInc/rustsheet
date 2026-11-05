"""Argument forms eval cells pass that the Gate A sampler never draws (T-3923): an evidenced df truncation,
BETA.DIST's and BETA.INV's [A, B] bounds, and RATE's guess."""

from __future__ import annotations

from typing import Any

import mpmath
import pytest

from rustsheet_tools.accuracy.defs.annuity import RATE
from rustsheet_tools.accuracy.defs.beta import BETA_DIST, BETA_INV, density, lower_tail
from rustsheet_tools.accuracy.defs.fisher import F_DIST, F_INV
from rustsheet_tools.accuracy.defs.gamma import CHISQ_INV, CHISQ_INV_RT
from rustsheet_tools.accuracy.defs.student_t import T_DIST
from rustsheet_tools.accuracy.eval_math import fit_args
from rustsheet_tools.accuracy.model import Definition, DomainError, Param
from rustsheet_tools.accuracy.tables import MathValue, compute


def _value(definition: Definition, args: list[Any]) -> MathValue:
    fitted = fit_args(definition, args)
    assert not isinstance(fitted, str), fitted
    return compute(definition, fitted)


def test_f_dist_reads_a_non_integer_df_truncated() -> None:
    # statistics.md: `F.DIST(2,150.7,3,FALSE)` 0.17202585847929508 is the density at df 150
    assert fit_args(F_DIST, [2.0, 150.7, 3.0, False]) == [2.0, 150, 3, False]
    assert _value(F_DIST, [2.0, 150.7, 3.0, False]) == _value(F_DIST, [2.0, 150.0, 3.0, False])
    assert fit_args(F_DIST, [2.0, 5.0, 89.1, True]) == [2.0, 5, 89, True]
    # a df that truncates below 1 is Excel's #NUM!: an error target, not a skip
    assert _value(F_DIST, [2.0, 0.5, 3.0, True]).why is not None


def test_t_dist_and_chisq_inv_rt_truncate_and_the_unevidenced_inverses_skip() -> None:
    assert fit_args(T_DIST, [2.0, 3.7, False]) == [2.0, 3, False]
    assert fit_args(CHISQ_INV_RT, [1e-9, 1.5]) == [1e-9, 1]
    assert fit_args(F_INV, [0.5, 2.5, 3.0]) == "non-integer for an integer parameter"
    assert fit_args(CHISQ_INV, [0.5, 2.5]) == "non-integer for an integer parameter"


def test_only_a_cited_parameter_truncates() -> None:
    every = [p for d in (F_DIST, T_DIST, CHISQ_INV_RT) for p in d.params if p.truncates]
    assert len(every) == 4
    assert all(p.truncates.startswith("statistics.md#") for p in every)


def test_beta_dist_with_bounds_is_the_standard_beta_at_the_exact_t() -> None:
    lo, hi, x = 0.95, 5.02, 1.49003654331704
    for cumulative in (True, False):
        bounded = _value(BETA_DIST, [x, 2.5, 3.0, cumulative, lo, hi]).value
        assert bounded is not None
        with mpmath.workdps(80):
            width = mpmath.mpf(hi) - mpmath.mpf(lo)
            t = (mpmath.mpf(x) - mpmath.mpf(lo)) / width
            a, b = mpmath.mpf(2.5), mpmath.mpf(3)
            want = lower_tail(t, a, b) if cumulative else density(t, a, b) / width
            assert mpmath.almosteq(mpmath.mpf(bounded), want, rel_eps=mpmath.mpf("1e-45"))
    # the defaults: A alone keeps B at 1, and [0, 1] is the plain call
    assert _value(BETA_DIST, [0.3, 2.0, 3.0, True, 0.0, 1.0]) == _value(BETA_DIST, [0.3, 2.0, 3.0, True])
    assert (
        _value(BETA_DIST, [0.625, 2.0, 3.0, True, 0.25]).value
        == _value(BETA_DIST, [0.5, 2.0, 3.0, True]).value
    )


def test_beta_dist_bounds_domain() -> None:
    assert _value(BETA_DIST, [6.0, 2.0, 3.0, True, 1.0, 5.0]).why is not None  # x above B
    assert _value(BETA_DIST, [2.0, 2.0, 3.0, True, 2.0, 2.0]).why is not None  # A = B
    # the density's given-x check: `BETA.DIST(0,0.5,2,FALSE,-1,1)` is #NUM! at an interior point
    assert _value(BETA_DIST, [0.0, 0.5, 2.0, False, -1.0, 1.0]).why is not None
    with pytest.raises(DomainError):
        BETA_DIST.check_domain([mpmath.mpf(1.5), mpmath.mpf(2), mpmath.mpf(3), True])


def test_beta_inv_with_bounds_maps_the_quantile() -> None:
    plain = _value(BETA_INV, [0.4998, 3.0, 600.0]).value
    bounded = _value(BETA_INV, [0.4998, 3.0, 600.0, 0.15, 1.2]).value
    assert plain is not None and bounded is not None
    want = mpmath.mpf(0.15) + (mpmath.mpf(1.2) - mpmath.mpf(0.15)) * mpmath.mpf(plain)
    assert mpmath.almosteq(mpmath.mpf(bounded), want, rel_eps=mpmath.mpf("1e-45"))
    assert _value(BETA_INV, [0.5, 2.0, 3.0, 1.2, 0.15]).why is not None  # B < A


def test_rate_ignores_its_guess() -> None:
    args = [12.0, -100.0, 1000.0, 0.0, 0.0]
    assert _value(RATE, [*args, 0.3]) == _value(RATE, args)
    assert fit_args(RATE, [*args, 0.3, 1.0]) == "argument count"


def test_call_parameters_are_never_drawn() -> None:
    for definition in (BETA_DIST, BETA_INV):
        names = [p.name for p in definition.params]
        assert "A" not in names and "B" not in names
        assert [p.name for p in definition.call_params] == ["A", "B"]
    # the sampler reads `params` only (sampler.py), so the Gate A tables do not change
    assert [p.name for p in RATE.call_params] == ["guess"]
    assert all(isinstance(p, Param) and p.optional for p in RATE.call_params)
