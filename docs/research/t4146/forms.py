"""T-4146: FV and PV in plain doubles against Excel's twice-rounded operators.

`fv(form, ...)` and `pv(form, ...)` evaluate the engine's expressions (annuity.rs `compute_future_value`, PV in
`fn_pv`): FV = -(pv*P + pmt*(k*((P-1)/r))), PV = -(fv + pmt*(k*((P-1)/r)))/P, with P Excel's `^` of Excel's 1+r.
Form "plain" rounds each operation once (the engine today); "twice" uses Excel's twice-rounded `+`, `*` and `/`
(`X87`), as RATE's residual does (`annuity_factor_rounded`). The negation is exact in both.
"""

from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels.solvers import caret_pow

FORMS = ("plain", "twice")


def _ops(form: str):
    if form == "twice":
        return X87.add, X87.mul, X87.div
    return (lambda a, b: a + b), (lambda a, b: a * b), (lambda a, b: a / b)


def _factor(form: str, r: float, p: float, t: float) -> float:
    add, mul, div = _ops(form)
    k = add(1.0, r) if t else 1.0
    return mul(k, div(add(p, -1.0), r))


def fv(form: str, r: float, n: float, pmt: float, pv: float, t: float) -> float:
    add, mul, _ = _ops(form)
    if r == 0:
        return -pv - pmt * n
    p = caret_pow(X87.add(1.0, r), n)
    return -add(mul(pv, p), mul(pmt, _factor(form, r, p, t)))


def pv(form: str, r: float, n: float, pmt: float, fv_: float, t: float) -> float:
    add, mul, div = _ops(form)
    if r == 0:
        return -fv_ - pmt * n
    p = caret_pow(X87.add(1.0, r), n)
    return div(-add(fv_, mul(pmt, _factor(form, r, p, t))), p)
