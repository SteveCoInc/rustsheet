"""T-3413 attempt 1: operation orders of T-2514's geometric principal sum (see fit.py).

Run: python3 docs/research/t3413/fit2.py (it imports fit.py from its own directory).
"""

from math import exp, expm1, log1p

from fit import main


def geoms(r: float, n: float, pv: float, s: int, e: int, t: int, p: float) -> dict[str, float]:
    """Orderings of `pmt * (1 - v^m) / r * (1+r) * v^(nper-end+1)`; type 1 also divides by 1+r."""
    del pv
    big_l = log1p(r)
    first = s if t == 0 else max(s, 2)
    m = e - first + 1
    a = -expm1(-m * big_l)
    tail = exp(-(n - e + 1) * big_l)
    near = exp(-(n - e) * big_l)
    g = 1 + r
    out = {
        "T2514": (p * (a / r * g)) * tail,
        "p*a/r*g*v": p * a / r * g * tail,
        "p*(a*g/r)*v": p * (a * g / r) * tail,
        "(p*a)*(g/r)*v": (p * a) * (g / r) * tail,
        "p*(a/(r/g))*v": p * (a / (r / g)) * tail,
        "p*(a/(1-1/g))*v": p * (a / (1 - 1 / g)) * tail,
        "p*(a/-expm1(-L))*v": p * (a / -expm1(-big_l)) * tail,
        "p*a*v/(r/g)": p * a * tail / (r / g),
        "p*(a/r*g*v)": p * (a / r * g * tail),
        "p*a/r*(g*v)": p * a / r * (g * tail),
        "p*a/r*exp(-(n-e)L)": p * a / r * near,
        "p*(a/r)*exp(-(n-e)L)": p * (a / r) * near,
        "p*a*exp(-(n-e)L)/r": p * a * near / r,
        "p/r*(exp(-(n-e)L)-exp(-(n-first+1)L))": p / r * (near - exp(-(n - first + 1) * big_l)),
        "p/r*exp(-(n-e)L)*a": p / r * near * a,
        "p*(exp(-(n-e)L)-exp(-(n-first+1)L))/r": p * (near - exp(-(n - first + 1) * big_l)) / r,
        "p*exp(-(n-e)L)*a/r": p * near * a / r,
    }
    if t == 1:
        out |= {k + "/g": v / g for k, v in out.items()}
    return out


if __name__ == "__main__":
    main(geoms, at_least=4)
