"""T-5524 fresh probe (decision 0188 R4.2): CHISQ.INV.RT and CHIINV at one degree of freedom, drawn after the
rule was frozen (seed 5524_2026_11_12).

The rule: Excel's value is the lower quantile of `(0.5 - p) + 0.5` formed with its twice-rounded `-` and `+`
(model.py `x87`), and `#NUM!` where that complement is 1. With k = p / 2^-53 the complement is 1 - n * 2^-53 for
an odd n only where k is within a quarter of n; the edge of `#NUM!` is k = 3/4 - 2^-13.

usage (from tools/): uv run python ../docs/research/t5524/fresh.py CASE.json PREDICTIONS.tsv

Sheets, none chosen by Excel's answer. Every p is a literal of at most 15 significant digits unless the cell
says otherwise, and the generator asserts that each reads back as the double the prediction used.
- Plain: p log-uniform over [1E-12, 0.25). The unconditional draw.
- Quarter: k an odd n plus or minus (1/4 + d), d = +-0.02 and +-0.05, p from 1E-12 to 1E-4: both sides of the
  quarter cuts the rule has, where `1 - p` has none.
- Half: k = n + 1/2 +- 0.03: both sides of the cut `1 - p` has, where the rule has none.
- Register: k an odd n plus or minus (1/4 + d), d = +-2^-14 and +-3 * 2^-14, p from 1E-12 to 1E-8. Within 2^-13
  of the quarter cut the 64-bit register lands `0.5 - p` on a tie of the store, so the twice-rounded form and the
  form in plain doubles differ on one side.
- Edge: the `#NUM!` edge 3 * 2^-55 - 2^-66 (k = 3/4 - 2^-13) and its neighbours, as 15-digit literals and as sums
  of powers of two, and the ends 2^-54 and 2^-53.
- Upper: p from 0.25 to 1 - 1E-12, where `0.5 - p` is exact, so the complement is the double `1 - p` itself.
- Names: CHIINV on rows of the sheets above, and degrees of freedom 1.5 and 1.99 (truncated to 1).
- Wide: the quarter cuts again at larger p, 1E-7 to 1E-3, with d = +-0.03 and +-0.06. Quarter and Half draw p over
  1E-12 to 1E-4 but keep a p only where its 15-digit literal lands within 1E-6 of a step of the k asked for, which
  no literal above about 2E-7 does: as drawn, Quarter's p run from 1.1E-12 to 1.7E-7, Half's to 8.4E-7 and
  Register's to 3.5E-9. Wide keeps a p within 0.01 of a step, which literals up to 1E-3 reach. It was added on
  2026-11-12, before any fill, from its own stream, so the seven sheets above keep their rows.

Added in review (CR-321929035 round 1), before any fill, each from its own stream:
- WideHalf: the half cut at the same larger p, k = n + 1/2 +- 0.03, six p in each decade from 1E-7 to 1E-3.
- Low: p between 2^-53 and 1E-12, where the engine has a second path (below 1E-12 it returns the math where
  Excel's value is more than 1E-4 off it): ten unconditional log-uniform p in each of the four decades.
- LowCuts: the cuts there, k = odd n +- (1/4 + d) for d = +-0.02 and +-0.05 and k = n + 1/2 +- 0.03, n from 3 to
  9007, the same count in each decade.
- Twins: `CHISQ.INV(1-n*2^-53,1)` beside six rows of each sheet, n the rule's whole number for that row, so
  that 1 - n * 2^-53 is the rule's complement exactly. The rule says each twin is its row's value bit for bit,
  whatever Excel's lower quantile does in its last digits.

Changed in review round 2, before any fill:
- Wide is redrawn with the decade, the side of n and d crossed (three cells in each of the sixteen combinations;
  the first draw gave each decade one side of the cut only), and the crossing is asserted.
- Twins takes Plain's rows with p from 0.001 up on which the rule differs from 1 - p first.
- LowNames: CHIINV on six rows of Low and six of LowCuts.

Also written: fresh-twins.tsv (each twin's row, n and complement) and fresh-math.tsv (the math's right-tail
quantile of p for every row, the rule's distance from it, and whether the engine returns the rule's value or,
past 1E-4 below p = 1E-12, the math).
"""

import json
import math
import pathlib
import random
import sys
from fractions import Fraction

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import model

rng = random.Random(5524_2026_11_12)
STEP = 2.0**-53


def literal(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def r15(v: float) -> float:
    return float(f"{v:.15g}")


def log_uniform(lo: float, hi: float) -> float:
    return r15(math.exp(rng.uniform(math.log(lo), math.log(hi))))


def at_steps(
    lo: float,
    hi: float,
    parity: int | None,
    offset: float,
    *,
    source: random.Random = rng,
    within: Fraction = Fraction(1, 10**6),
) -> float:
    """A 15-digit p whose k is n + offset to `within` of a step, n drawn log-uniformly over [lo, hi] / 2^-53 with
    the given parity."""
    for _ in range(10_000):
        n = int(math.exp(source.uniform(math.log(lo / STEP), math.log(hi / STEP))))
        if parity is not None and n % 2 != parity:
            n += 1
        p = r15((n + offset) * STEP)
        if abs(model.steps(p) - (n + Fraction(offset))) < within:
            return p
    raise AssertionError((lo, hi, parity, offset))


def plain(count: int) -> list[str]:
    return [literal(log_uniform(1e-12, 0.25)) for _ in range(count)]


def quarter(count: int) -> list[str]:
    rows = []
    for i in range(count):
        side = 1 if i % 2 else -1
        d = (0.02, -0.02, 0.05, -0.05)[(i // 2) % 4]
        rows.append(literal(at_steps(1e-12, 1e-4, 1, side * (0.25 + d))))
    return rows


def half(count: int) -> list[str]:
    return [literal(at_steps(1e-12, 1e-4, None, 0.5 + (0.03 if i % 2 else -0.03))) for i in range(count)]


def register(count: int) -> list[str]:
    rows = []
    for i in range(count):
        side = 1 if i % 2 else -1
        d = (2.0**-14, -(2.0**-14), 3 * 2.0**-14, -3 * 2.0**-14)[(i // 2) % 4]
        rows.append(literal(at_steps(1e-12, 1e-8, 1, side * (0.25 + d))))
    return rows


wide_rng = random.Random(5524_2026_11_13)


def wide(count: int) -> list[str]:
    rows = []
    for i in range(count):
        # The decade, the side of n and d are crossed: i % 4, (i // 4) % 2 and (i // 8) % 4 take every combination
        # three times in 48 rows. (As first drawn the three were all functions of i % 4, so each decade had one
        # side of the cut only: review round 2.)
        lo = 10.0 ** (-7 + i % 4)
        side = 1 if (i // 4) % 2 else -1
        d = (0.03, -0.03, 0.06, -0.06)[(i // 8) % 4]
        p = at_steps(lo, 10 * lo, 1, side * (0.25 + d), source=wide_rng, within=Fraction(1, 100))
        rows.append(literal(p))
    return rows


EDGE = [
    # 15-digit literals across the edge, 8.32531743197307E-17
    "8.3253E-17",
    "8.32531E-17",
    "8.325317E-17",
    "8.3253174E-17",
    "8.32531743E-17",
    "8.32531744E-17",
    "8.3253175E-17",
    "8.325318E-17",
    "8.32532E-17",
    "8.3254E-17",
    "8.3266E-17",
    "8.3267E-17",
    # sums of powers of two: the edge itself, one double either side, and the two ties around it
    "3*2^-55-2^-66",
    "3*2^-55-2^-66-2^-106",
    "3*2^-55-2^-66+2^-106",
    "3*2^-55-2^-65",
    "3*2^-55-2^-67",
    "3*2^-55-3*2^-67",
    "3*2^-55",
    "3*2^-55+2^-106",
    "2^-54",
    "2^-54+2^-106",
    "2^-53",
    "2^-53-2^-106",
]


half_wide_rng = random.Random(5524_2026_11_14)
low_rng = random.Random(5524_2026_11_15)
low_cut_rng = random.Random(5524_2026_11_16)
G = 1e-4  # decision 0188 R3's gap
ENGINE_EXACT_P_MIN = 1e-12  # from here up the engine never asks the math (chisq.rs)
LOW_DECADES = ((1.2e-16, 1e-15), (1e-15, 1e-14), (1e-14, 1e-13), (1e-13, 1e-12))
# The cuts start at n = 3: the cuts of n = 1 are the Edge sheet's.
LOW_CUT_DECADES = ((3e-16, 1e-15), *LOW_DECADES[1:])


def wide_half(count: int) -> list[str]:
    rows = []
    for i in range(count):
        lo = 10.0 ** (-7 + i % 4)
        offset = 0.5 + (0.03 if (i // 4) % 2 else -0.03)
        rows.append(
            literal(at_steps(lo, 10 * lo, None, offset, source=half_wide_rng, within=Fraction(1, 100)))
        )
    return rows


def low(count: int) -> list[str]:
    rows = []
    for i in range(count):
        lo, hi = LOW_DECADES[i % 4]
        rows.append(literal(r15(math.exp(low_rng.uniform(math.log(lo), math.log(hi))))))
    return rows


def low_cuts(quarters: int, halves: int) -> list[str]:
    rows = []
    for i in range(quarters):
        side = 1 if i % 2 else -1
        d = (0.02, -0.02, 0.05, -0.05)[(i // 2) % 4]
        lo, hi = LOW_CUT_DECADES[(i // 8) % 4]
        rows.append(literal(at_steps(lo, hi, 1, side * (0.25 + d), source=low_cut_rng)))
    for i in range(halves):
        lo, hi = LOW_CUT_DECADES[(i // 2) % 4]
        rows.append(literal(at_steps(lo, hi, None, 0.5 + (0.03 if i % 2 else -0.03), source=low_cut_rng)))
    return rows


def crossing(arguments: list[str]) -> dict[tuple[int, bool, bool], int]:
    """Cells per (decade of p, the rule differs from 1 - p, k above its nearest odd whole number)."""
    counts: dict[tuple[int, bool, bool], int] = {}
    for argument in arguments:
        p = float(argument)
        k = model.steps(p)
        odd = 2 * math.floor(k / 2) + 1
        key = (math.floor(math.log10(p)), model.x87(p) != model.plain(p), k > odd)
        counts[key] = counts.get(key, 0) + 1
    return counts


def per_decade(arguments: list[str]) -> dict[int, int]:
    counts: dict[int, int] = {}
    for argument in arguments:
        decade = math.floor(math.log10(float(argument)))
        counts[decade] = counts.get(decade, 0) + 1
    return counts


def upper(count: int) -> list[str]:
    rows = [literal(log_uniform(0.25, 0.75)) for _ in range(count // 2)]
    return rows + [literal(r15(1 - log_uniform(1e-12, 0.25))) for _ in range(count - count // 2)]


def twin_rows(sheets: dict[str, list[str]]) -> tuple[dict[str, str], list[str]]:
    """The Twins sheet and its table: the rule's complement written out, beside six rows of each sheet (Plain's
    below 0.1, so that n is a whole number of at most 15 digits) and beside the edge. On Plain the rows where the
    15-digit read is weakest come first: p from 0.001 up on which the rule differs from 1 - p."""
    paired = [("Edge", "A13", model.argument(EDGE[12]))]
    for name in ("Plain", "Quarter", "Half", "Register", "Wide", "WideHalf", "Low", "LowCuts"):
        below = [(f"A{i}", float(a)) for i, a in enumerate(sheets[name], start=1) if float(a) < 0.1]
        weak = [(a, p) for a, p in below if name == "Plain" and p >= 0.001 and model.x87(p) != model.plain(p)]
        chosen = (weak + [row for row in below if row not in weak])[: max(6, len(weak))]
        paired += [(name, address, p) for address, p in chosen]
    cells: dict[str, str] = {}
    table = ["cell\ttwin of\tn\tcomplement\tquantile"]
    for i, (name, address, p) in enumerate(paired, start=1):
        u = model.x87(p)
        n = (1 - Fraction(u)) / Fraction(STEP)
        assert n.denominator == 1 and 0 < n < 10**15 and 1.0 - float(n) * STEP == u, (name, address)
        cells[f"A{i}"] = f"=CHISQ.INV(1-{n.numerator}*2^-53,1)"
        table.append(f"Twins!A{i}\t{name}!{address}\t{n.numerator}\t{u!r}\t{model.quantile(u)!r}")
    return cells, table


def math_rows(rows: list[tuple[str, str, str, float]]) -> list[str]:
    """Per row: the math's right-tail quantile of p, the rule's relative distance from it, and what the engine
    returns: the rule's value, or the math where that is more than G off below p = 1E-12."""
    lines = ["sheet\tcell\tmath\trule from the math\tthe engine returns"]
    for name, address, _argument, p in rows:
        rule = model.quantile(model.x87(p))
        if rule is None:
            lines.append(f"{name}\t{address}\t\t\t#NUM!")
            continue
        exact = model.right_tail_quantile(p)
        gap = abs(rule - exact) / exact
        returns = "the math" if p < ENGINE_EXACT_P_MIN and gap > G else "the rule's value"
        lines.append(f"{name}\t{address}\t{exact!r}\t{gap:.3e}\t{returns}")
    return lines


def main() -> None:
    case_path, predictions_path = sys.argv[1], sys.argv[2]
    sheets = {
        "Plain": plain(80),
        "Quarter": quarter(64),
        "Half": half(32),
        "Register": register(32),
        "Edge": EDGE,
        "Upper": upper(20),
    }
    cells: dict[str, dict[str, str]] = {name: {} for name in sheets}
    rows = []
    for name, arguments in sheets.items():
        for i, argument in enumerate(arguments, start=1):
            p = model.argument(argument)
            if name != "Edge":
                assert (
                    literal(p) == argument and len(argument.split("E")[0].replace(".", "").lstrip("0")) <= 15
                )
            cells[name][f"A{i}"] = f"=CHISQ.INV.RT({argument},1)"
            rows.append((name, f"A{i}", argument, p))
    names: dict[str, str] = {}
    picks = [(n, a) for n in ("Plain", "Quarter", "Half", "Register", "Edge") for a in sheets[n][:6]]
    for i, (_sheet, argument) in enumerate(picks, start=1):
        p = model.argument(argument)
        names[f"A{i}"] = f"=CHIINV({argument},1)"
        rows.append(("Names", f"A{i}", argument, p))
    for i, (argument, df) in enumerate(
        [(a, d) for a in sheets["Quarter"][:6] for d in ("1.5", "1.99")], start=1
    ):
        names[f"B{i}"] = f"=CHISQ.INV.RT({argument},{df})"
        rows.append(("Names", f"B{i}", argument, float(argument)))
    cells["Names"] = names
    later = {"Wide": wide(48), "WideHalf": wide_half(24), "Low": low(40), "LowCuts": low_cuts(32, 16)}
    assert per_decade(later["Wide"]) == dict.fromkeys(range(-7, -3), 12)
    # Three cells on each side of the cut, on each side of n, in each decade.
    assert crossing(later["Wide"]) == {
        (decade, outside, above): 3
        for decade in range(-7, -3)
        for outside in (False, True)
        for above in (False, True)
    }
    assert per_decade(later["WideHalf"]) == dict.fromkeys(range(-7, -3), 6)
    assert per_decade(later["Low"]) == dict.fromkeys(range(-16, -12), 10)
    assert per_decade(later["LowCuts"]) == dict.fromkeys(range(-16, -12), 12)
    for name, arguments in later.items():
        cells[name] = {}
        for i, argument in enumerate(arguments, start=1):
            p = float(argument)
            assert literal(p) == argument and len(argument.split("E")[0].replace(".", "").lstrip("0")) <= 15
            cells[name][f"A{i}"] = f"=CHISQ.INV.RT({argument},1)"
            rows.append((name, f"A{i}", argument, p))
    sheets.update(later)
    cells["Twins"], twins = twin_rows(sheets)
    pathlib.Path(predictions_path).with_name("fresh-twins.tsv").write_text("\n".join(twins) + "\n")
    # CHIINV between 2^-53 and 1E-12, where four of its nine cells outside the envelope sit.
    cells["LowNames"] = {}
    low_rows = [argument for name in ("Low", "LowCuts") for argument in sheets[name][:6]]
    for i, argument in enumerate(low_rows, start=1):
        cells["LowNames"][f"A{i}"] = f"=CHIINV({argument},1)"
        rows.append(("LowNames", f"A{i}", argument, float(argument)))
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-5524 fresh probe (decision 0188 R4.2), drawn after the rule was frozen: CHISQ.INV.RT and CHIINV at one "
            "degree of freedom are the lower quantile of (0.5 - p) + 0.5 formed with Excel's twice-rounded - and +, "
            "and #NUM! where that complement is 1. Predictions in docs/research/t5524/fresh-predictions.tsv, "
            "committed before the fill. Plain: p log-uniform over [1E-12, 0.25). Quarter and Half: p on both sides "
            "of the cuts the rule has and of the one 1 - p has. Register: p within 2^-13 of 2^-53 of a quarter cut, "
            "where the 64-bit register decides. Edge: the #NUM! edge 3*2^-55 - 2^-66. Upper: p from 0.25 up, where "
            "the complement is 1 - p. Names: CHIINV, and degrees of freedom 1.5 and 1.99. Wide (added before the "
            "fill): the quarter cuts at p from 1E-7 to 1E-3. Added in review, before the fill: WideHalf, the half "
            "cut at those p; Low, p between 2^-53 and 1E-12; LowCuts, the cuts there; Twins, CHISQ.INV at the rule's "
            "complement written as 1-n*2^-53 beside six rows of each sheet; LowNames, CHIINV on rows of Low and "
            "LowCuts."
        ),
        "tags": ["M9", "statistical", "probe", "T-5524"],
        "sheets": [{"name": name, "cells": cells[name]} for name in cells],
        "expected": {},
    }
    pathlib.Path(case_path).write_text(json.dumps(case, indent=1) + "\n")
    lines = [
        "sheet\tcell\tp\tsteps\tcomplement 1-p\tcomplement doubles\tcomplement rule\tquantile 1-p\tquantile doubles\tquantile rule"
    ]
    for name, address, argument, p in rows:
        complements = [rule(p) for rule in model.RULES.values()]
        quantiles = [model.quantile(u) for u in complements]
        lines.append(
            "\t".join(
                [name, address, argument, f"{float(model.steps(p)):.6f}"]
                + [repr(u) for u in complements]
                + ["#NUM!" if x is None else repr(x) for x in quantiles]
            )
        )
    pathlib.Path(predictions_path).write_text("\n".join(lines) + "\n")
    pathlib.Path(predictions_path).with_name("fresh-math.tsv").write_text("\n".join(math_rows(rows)) + "\n")


main()
