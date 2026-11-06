"""The CUMPRINC/CUMIPMT reference of fitlab/kernels/cumulative.py (T-3524)."""

from __future__ import annotations

import mpmath

from rustsheet_tools.fitlab.kernels import cumulative, registry


def _exact(function: str, *args: float) -> float:
    with mpmath.workdps(60):
        return float(cumulative.exact_mp(function, args))


def test_whole_loan_principal_is_the_loan() -> None:
    for period_type in (0.0, 1.0):
        assert abs(_exact("CUMPRINC", 0.1, 2, 100, 1, 2, period_type) + 100) < 1e-12


def test_interest_is_payments_less_principal() -> None:
    payment = 100 * 0.1 / (1 - 1.1**-2)  # 57.6190476...
    assert abs(_exact("CUMIPMT", 0.1, 2, 100, 1, 2, 0.0) + (2 * payment - 100)) < 1e-12
    # Type 1 pays at the start: period 1 pays no interest, the payment is over 1+r.
    assert _exact("CUMIPMT", 0.1, 2, 100, 1, 1, 1.0) == 0.0
    assert abs(_exact("CUMIPMT", 0.1, 2, 100, 2, 2, 1.0) + (payment / 1.1 - payment / 1.1 / 1.1)) < 1e-12


def test_one_period_principal() -> None:
    # Period k's principal is pmt·v^(nper-k+1): the last period's is pmt/(1+r).
    payment = 100 * 0.1 / (1 - 1.1**-2)
    assert abs(_exact("CUMPRINC", 0.1, 2, 100, 2, 2, 0.0) + payment / 1.1) < 1e-12


def test_fits_are_registered() -> None:
    assert {"cumulative", "cumulative-fresh"} <= set(registry())
