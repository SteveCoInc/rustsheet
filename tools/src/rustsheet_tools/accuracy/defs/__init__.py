"""Gate A definitions by family; importing this package registers every definition."""

from rustsheet_tools.accuracy.defs import (
    annuity,
    beta,
    cashflow,
    discrete,
    fisher,
    gamma,
    inverse_trig,
    rounding,
    student_t,
    trig,
)

__all__ = [
    "annuity",
    "beta",
    "cashflow",
    "discrete",
    "fisher",
    "gamma",
    "inverse_trig",
    "rounding",
    "student_t",
    "trig",
]
