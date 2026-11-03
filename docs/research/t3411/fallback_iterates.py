"""T-3411: are Excel's IRR returns, where the secant fails, iterates of Newton on P(v) from v = 0?

For every IRR cell with an oracle value in eval/suites, this replays the engine's secant (`irr_secant`: d = 1 - v,
NPV as `discounted_sum` at x = 1/v with Excel's twice-rounded operators, 50 steps, retry from 0.1) and keeps the cells
where both runs fail. For those it runs `irr_fallback`'s Newton on P(v) = sum c_i v^i (plain doubles, Horner) from
v = 0 without its cap, and reports whether Excel's value is 1/v - 1 at some iterate, and where: at the engine's stop
(the first step under 1e-7 max(|v|, 1)), one iterate later, or elsewhere.

The secant replay is exact rational arithmetic rounded to 64 then 53 bits; it differs from the engine on 7 of the
703 cells it keeps (a root where the engine's run fails). Stdlib only, a few minutes:

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


def secant_finds_root(flows: list[Fraction], guess: float) -> bool:
    """Whether the secant run decides the call (a root, or `#DIV/0!` at v = 0)."""

    def at(d: Fraction) -> Fraction | None:
        v = plain(1 - d)
        return npv(flows, v) if v > 0 else None

    g = Fraction(guess)
    d0 = twice(g / plain(1 + g))
    f0 = at(d0)
    if f0 is None:
        return False
    d1 = plain(d0 - (Fraction(1e-3) if f0 > 0 else -Fraction(1e-3)))
    f1 = at(d1)
    largest = max(abs(c) for c in flows)
    for _ in range(50):
        if f1 is None or f1 == f0:
            return False
        d2 = plain(d1 - twice(plain(f1 * plain(d1 - d0)) / plain(f1 - f0)))
        if d2 >= 1:
            # d2 = 1 is `#DIV/0!` without the retry or the fallback: not a fallback cell
            return d2 == 1
        f2 = at(d2)
        if f2 is not None and abs(d2 - d1) < Fraction(1e-7):
            stalled = d2 == d1 and abs(f2) < Fraction(1e-7) * largest
            if abs(f2) < Fraction(1e-7) or stalled:
                return float(twice(1 / plain(1 - d2))) < 1e7
        d0, f0, d1, f1 = d1, f1, d2, f2
    return False


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
                if secant_finds_root(exact, guess) or (guess != 0.1 and secant_finds_root(exact, 0.1)):
                    continue
                if cell["t"] != "n":
                    tally[f"secant fails, Excel {cell['v']}"] += 1
                    continue
                its = newton_iterates(values)
                hit = next((k for k, (v, _) in enumerate(its) if v > 0 and rate(v) == cell["v"]), None)
                stop = next((k for k, (v, s) in enumerate(its) if s < 1e-7 * max(abs(v), 1)), None)
                if hit is None:
                    kind = "no iterate"
                elif stop is not None and rate(its[stop][0]) == cell["v"]:
                    kind = "iterate at the engine's stop"
                elif stop is not None and stop + 1 < len(its) and rate(its[stop + 1][0]) == cell["v"]:
                    kind = "iterate one after the engine's stop"
                else:
                    kind = "another iterate"
                tally[f"secant fails, Excel returns: {kind}"] += 1
                print(case.stem.removesuffix(".case"), addr, kind, hit, stop, flush=True)
    for key, count in sorted(tally.items()):
        print(f"{count:5d}  {key}")


if __name__ == "__main__":
    main()
