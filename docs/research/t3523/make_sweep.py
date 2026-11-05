"""T-3523: fresh sweep (decision 0188 R4.2) over the domains where IPMT's DISCOUNTED_INTEREST_FROM selector
can still run: T-2243's discounted split declines there (a subnormal or tiny rate makes the payment factor non-normal,
a negative rate's (1+rate)^-nper overflows, or a product overflows), so `period_payments` falls back to the
exact balance and the selector. Log-uniform draws from a fixed seed; no earlier fit saw these cells.

Run from the repository root: python3 docs/research/t3523/make_sweep.py
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import lit, write_checked_case  # noqa: E402 (sweep_case is found through the sys.path entry above)

OUT = Path("eval/suites/probes-numeric/cases/t3523-ipmt-selector-sweep.case.json")


def logu(rng: random.Random, lo: float, hi: float) -> float:
    """A log-uniform draw from [lo, hi]."""
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def main() -> None:
    """Write the sweep case: column A negative rates past the power's range, B subnormal rates, C huge values,
    D tiny normal rates."""
    rng = random.Random(3523)
    cells: dict[str, str] = {}

    for i in range(1, 201):
        # A: rate -0.9..-0.01, nper past where (1+rate)^-nper overflows (log(1.8e308)/-log1p(rate) .. x8).
        rate = -logu(rng, 0.01, 0.9)
        limit = 709.8 / -math.log1p(rate)
        nper = math.ceil(limit * rng.uniform(1.0, 8.0))
        period = max(1, min(nper, nper - math.floor(logu(rng, 1, nper))))
        pv = logu(rng, 1, 1e6) * rng.choice((1, -1))
        fv = rng.choice((0.0, logu(rng, 1, 1e6)))
        typ = rng.choice((0, 1))
        cells[f"A{i}"] = f"=IPMT({lit(rate)},{period},{nper},{lit(pv)},{lit(fv)},{typ})"
        # B: subnormal rates of either sign. A literal below the normal range is not a valid Excel number
        # and a power of two below it flushes to 0 (numbers.md "Subnormal results become 0"), but the
        # difference of two normal literals keeps its subnormal value (arrays.md), so the rate is written
        # as one: `(aE-308-bE-308)` with a - b below 2.2250738585072014E-308.
        while True:
            a = round(rng.uniform(2.3, 9.9), 6)
            b = round(a - rng.uniform(0.01, 2.2), 6)
            if 0 < float(f"{a}E-308") - float(f"{b}E-308") < 2.2250738585072014e-308:
                break
        sign = rng.choice(("", "-"))
        nper = math.ceil(logu(rng, 2, 1e6))
        period = math.ceil(rng.uniform(1, nper))
        pv = logu(rng, 1, 9.9e307) * rng.choice((1, -1))
        fv = rng.choice((0.0, logu(rng, 1, 9.9e307)))
        typ = rng.choice((0, 1))
        cells[f"B{i}"] = f"=IPMT({sign}({a:.6f}E-308-{b:.6f}E-308),{period},{nper},{lit(pv)},{lit(fv)},{typ})"
        # C: values near the double range at ordinary rates, up to 9.9E+307 (the largest literal Excel reads).
        rate = logu(rng, 1e-3, 10) * rng.choice((1, 1, -0.05))
        nper = math.ceil(logu(rng, 2, 400))
        period = math.ceil(rng.uniform(1, nper))
        pv = logu(rng, 1e290, 9.9e307)
        fv = rng.choice((0.0, logu(rng, 1e290, 9.9e307)))
        typ = rng.choice((0, 1))
        cells[f"C{i}"] = f"=IPMT({lit(rate)},{period},{nper},{lit(pv)},{lit(fv)},{typ})"
        # D: normal rates too small for the payment factor (1E-300 to 1E-12, either sign) over long terms
        # and values up to 9.9E+307: the discounted split declines here too.
        rate = logu(rng, 1e-300, 1e-12) * rng.choice((1, -1))
        nper = math.ceil(logu(rng, 2, 1e9))
        period = math.ceil(rng.uniform(1, nper))
        pv = logu(rng, 1, 9.9e307) * rng.choice((1, -1))
        fv = rng.choice((0.0, logu(rng, 1, 9.9e307)))
        typ = rng.choice((0, 1))
        cells[f"D{i}"] = f"=IPMT({lit(rate)},{period},{nper},{lit(pv)},{lit(fv)},{typ})"
    write_checked_case(
        OUT,
        "T-3523 fresh sweep (decision 0188 R4.2): IPMT where T-2243's discounted split declines and the "
        "DISCOUNTED_INTEREST_FROM selector can run (negative rates past the power's range, subnormal rates, "
        "values near the double range, tiny normal rates).",
        ["M9", "financial", "probe", "T-3523"],
        cells,
    )
    print(len(cells))


if __name__ == "__main__":
    main()
