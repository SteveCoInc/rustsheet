"""T-3411: are Excel's IRR returns, where the secant fails, iterates of Newton on P(v) from v = 0?

For every IRR cell with an oracle value in eval/suites, this replays the engine's secant (`irr_secant`: d = 1 - v,
NPV as `discounted_sum` at x = 1/v with Excel's twice-rounded operators, 50 steps, retry from 0.1) and keeps the cells
where both runs fail. For those it runs `irr_fallback`'s Newton on P(v) = sum c_i v^i (plain doubles, Horner) from
v = 0 without its cap, and reports whether Excel's value is 1/v - 1 at some iterate, and where: at the engine's stop
(the first step under 1e-7 max(|v|, 1)), one iterate later, both (the two give the same rate, so the cell cannot
tell them apart) or elsewhere. It also tallies the cells by case, by how the last secant run fails, and, on the
cells where Excel is an error, whether that Newton search converges to a root under the far-root bound.

The secant replay is exact rational arithmetic rounded to 64 then 53 bits (a few cells may differ from the engine
in the last bit of a step). The rounding mirrors tools' `fitlab.exact.rounded_to`, kept here so the script is
stdlib only. A few minutes:

    python3 -I docs/research/t3411/fallback_iterates.py <repo>
"""

from __future__ import annotations

import collections
import json
import math
import re
import sys
from fractions import Fraction
from pathlib import Path

LIMIT = Fraction(2) ** 1024
TINY = Fraction(2) ** -1022


def rounded(q: Fraction, bits: int) -> Fraction:
    """q rounded to `bits` significant bits, ties to even (no exponent range)."""
    if q == 0:
        return q
    a = abs(q)
    e = a.numerator.bit_length() - a.denominator.bit_length()
    if Fraction(2) ** e > a:
        e -= 1
    elif Fraction(2) ** (e + 1) <= a:
        e += 1
    scaled = a * Fraction(2) ** (bits - 1 - e)
    n, r = divmod(scaled.numerator, scaled.denominator)
    rest = Fraction(r, scaled.denominator)
    if rest > Fraction(1, 2) or (rest == Fraction(1, 2) and n & 1):
        n += 1
    out = Fraction(n) * Fraction(2) ** (e - bits + 1)
    return out if q > 0 else -out


def twice(q: Fraction) -> Fraction:
    return rounded(rounded(q, 64), 53)


def plain(q: Fraction) -> Fraction:
    return rounded(q, 53)


def npv(flows: list[Fraction], v: Fraction) -> Fraction | None:
    x = twice(1 / v)
    total, discount = Fraction(0), Fraction(1)
    for i, c in enumerate(flows):
        if i:
            discount = twice(discount * x)
        if discount == 0 or not TINY <= abs(discount) < LIMIT:
            return None
        total = twice(total + twice(c / discount))
    return total if abs(total) < LIMIT else None


def secant_run(flows: list[Fraction], guess: float) -> str:
    """How one secant run ends: "decided" (a root, or `#DIV/0!` at v = 0, which skips the retry and the fallback),
    or the way it fails: "flat", "cap" or "iterate" (outside (0, 1) in d, or an NPV out of range)."""

    def at(d: Fraction) -> Fraction | None:
        v = plain(1 - d)
        return npv(flows, v) if v > 0 else None

    g = Fraction(guess)
    d0 = twice(g / plain(1 + g))
    f0 = at(d0)
    if f0 is None:
        return "iterate"
    d1 = plain(d0 - (Fraction(1e-3) if f0 > 0 else -Fraction(1e-3)))
    f1 = at(d1)
    largest = max(abs(c) for c in flows)
    for _ in range(50):
        if f1 is None:
            return "iterate"
        if f1 == f0:
            return "flat"
        d2 = plain(d1 - twice(plain(f1 * plain(d1 - d0)) / plain(f1 - f0)))
        if d2 >= 1:
            return "decided" if d2 == 1 else "iterate"
        f2 = at(d2)
        if f2 is not None and abs(d2 - d1) < Fraction(1e-7):
            stalled = d2 == d1 and abs(f2) < Fraction(1e-7) * largest
            if abs(f2) < Fraction(1e-7) or stalled:
                return "decided" if float(twice(1 / plain(1 - d2))) < 1e7 else "iterate"
        d0, f0, d1, f1 = d1, f1, d2, f2
    return "cap"


def newton_iterates(flows: list[float], cap: int = 5000) -> list[tuple[float, float]]:
    """(iterate, |step|) of Newton on P(v) from v = 0, as `irr_fallback` computes it, without its cap."""
    out, v = [], 0.0
    for _ in range(cap):
        p = dp = 0.0
        for c in reversed(flows):
            dp = dp * v + p
            p = p * v + c
        if dp == 0:
            break
        nxt = v - p / dp
        if not math.isfinite(nxt):
            break
        out.append((nxt, abs(nxt - v)))
        if nxt == v:
            break
        v = nxt
    return out


def rate(v: float) -> float:
    return float(twice(1 / Fraction(v))) - 1.0


def classify(its: list[tuple[float, float]], stop: int | None, value: float) -> str:
    """Which iterate Excel's `value` is, relative to the engine's stop (Excel's value is some iterate)."""
    if stop is None:
        return "another iterate"
    at_stop = its[stop][0] > 0 and rate(its[stop][0]) == value
    after = stop + 1 < len(its) and its[stop + 1][0] > 0
    at_next = after and rate(its[stop + 1][0]) == value
    if at_stop and (at_next or not after):
        return "the iterate at the engine's stop and the next one (same rate): ambiguous"
    if at_stop:
        return "the iterate at the engine's stop, not the next one"
    if at_next:
        return "the iterate one after the engine's stop, not the one at it"
    return "another iterate"


def main() -> None:
    repo = Path(sys.argv[1])
    tally: collections.Counter[str] = collections.Counter()
    for case in sorted(repo.glob("eval/suites/*/cases/*.case.json")):
        text = case.read_text()
        oracle = case.with_name(case.name.replace(".case.", ".oracle."))
        if "=IRR(" not in text or not oracle.exists():
            continue
        sheets = json.loads(oracle.read_text())["sheets"]
        for sheet in json.loads(text)["sheets"]:
            for addr, formula in sheet["cells"].items():
                m = re.match(r"=IRR\(\{([^}]*)\}(?:,([^)]*))?\)$", str(formula))
                cell = sheets.get(sheet["name"], {}).get(addr)
                if not m or cell is None:
                    continue
                try:
                    values = [float(t) for t in m.group(1).split(",")]
                    guess = float(m.group(2)) if m.group(2) else 0.1
                except ValueError:
                    continue  # a text, logical or empty element or guess: not a numeric series
                while values and values[0] == 0:
                    values = values[1:]
                if not -1 < guess < 2.0**53 or all(c >= 0 for c in values) or all(c <= 0 for c in values):
                    continue  # `#VALUE!` or `#NUM!` before any search
                exact = [Fraction(c) for c in values]
                end = secant_run(exact, guess)
                if end != "decided" and guess != 0.1:
                    end = secant_run(exact, 0.1)
                if end == "decided":
                    continue
                name = case.stem.removesuffix(".case")
                excel = "returns" if cell["t"] == "n" else cell["v"]
                tally[f"case {name}: secant fails"] += 1
                tally[f"last run {end}, Excel {excel}"] += 1
                its = newton_iterates(values)
                stop = next((k for k, (v, s) in enumerate(its) if s < 1e-7 * max(abs(v), 1)), None)
                if cell["t"] != "n":
                    root = its[stop][0] if stop is not None else None
                    converged = root is not None and root > 0 and float(twice(1 / Fraction(root))) < 1e7
                    tally[f"Excel {excel}, Newton {'converges' if converged else 'fails'}"] += 1
                    continue
                hit = next((k for k, (v, _) in enumerate(its) if v > 0 and rate(v) == cell["v"]), None)
                kind = classify(its, stop, cell["v"]) if hit is not None else "no iterate"
                tally[f"secant fails, Excel returns {kind}"] += 1
                step = its[stop][1] if stop is not None else math.nan
                print(name, addr, kind, "hit", hit, "stop", stop, f"step {step:.3g}", flush=True)
    for key, count in sorted(tally.items()):
        print(f"{count:5d}  {key}")


if __name__ == "__main__":
    main()
