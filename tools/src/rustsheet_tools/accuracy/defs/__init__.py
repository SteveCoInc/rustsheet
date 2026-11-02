"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import (
    aggregate,
    annuity,
    beta,
    cashflow,
    discrete,
    fisher,
    gamma,
    hyperbolic,
    inverse_trig,
    rounding,
    student_t,
    trig,
)

__all__ = [
    "aggregate",
    "annuity",
    "beta",
    "cashflow",
    "discrete",
    "fisher",
    "gamma",
    "hyperbolic",
    "inverse_trig",
    "rounding",
    "student_t",
    "trig",
]
