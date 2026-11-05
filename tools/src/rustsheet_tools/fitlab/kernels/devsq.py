"""DEVSQ's summation form over every eval cell it owns and its fresh sweep (T-1749, decision 0188 R4).

DEVSQ is Σ(x - mean)² around the double mean (two passes), not the one-pass Σx² - (Σx)²/n the
engine had: a choice between structural forms, so it owes the hold-out and a fresh sweep (R4.1,
R4.2) but no `FITTED` tag. Scored: the two forms and the correctly rounded Σ(x - mean)² of the
double inputs, on the owned gen-m9 and probes cells (`devsq`) and on the sweep generated and
filled after the form was chosen (`devsq-fresh`, probes-numeric/t1749-devsq-fresh). The margin,
5 points, was stated before scoring. Results: T-1749's Outcome and statistics.md "DEVSQ is a
two-pass sum (T-1749)".
"""

from __future__ import annotations

from fractions import Fraction

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.holdout import REFERENCE
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg

_OWNED = [
    "gen-m9/05d44f6dd6cfc567",
    "gen-m9/06b4e8f5de409d5e",
    "gen-m9/0a614cfe974b59c3",
    "gen-m9/0a9226016e8f5f33",
    "gen-m9/146b46e74f2dac3b",
    "gen-m9/165db58d1540f014",
    "gen-m9/27601819397dd56e",
    "gen-m9/2801c1c11c21e134",
    "gen-m9/285d87cea1aaa73c",
    "gen-m9/2a18b2f5a4dc43ac",
    "gen-m9/320d51765af9b6b7",
    "gen-m9/35e8168aae2b00f8",
    "gen-m9/3a25a03eb94fcfbd",
    "gen-m9/3e98b26cf6713e77",
    "gen-m9/3ec66461400faf2f",
    "gen-m9/40425341c5034eed",
    "gen-m9/4250f880ab640c49",
    "gen-m9/4ab2ba98d26c8a5e",
    "gen-m9/4bad303d32cc3f46",
    "gen-m9/4e55500bb8ad3cd8",
    "gen-m9/530c636749a85209",
    "gen-m9/589f42e67f4b4b8d",
    "gen-m9/58e3253bbd3a7a82",
    "gen-m9/60b0b6e69ad35a45",
    "gen-m9/623663cacea53bbc",
    "gen-m9/68d8ffb69ff739d8",
    "gen-m9/7674c89c824aca43",
    "gen-m9/7bd219ec0c0c5e22",
    "gen-m9/7c20351562a2d71d",
    "gen-m9/85f48d5cdfb24555",
    "gen-m9/899f9555e67da2d6",
    "gen-m9/8d3588d8df0812df",
    "gen-m9/8dd2312c96effb1b",
    "gen-m9/8f2ac25945af87da",
    "gen-m9/8fec7b207f4dd753",
    "gen-m9/9cca17438a7e6f8a",
    "gen-m9/a013a45a8fbcab4f",
    "gen-m9/a54c738c697617c3",
    "gen-m9/a5828d3925575713",
    "gen-m9/a612e89321a6df9c",
    "gen-m9/a61e547a1cab994c",
    "gen-m9/a790a4d4a6a82756",
    "gen-m9/a7bf0d8d81d33ba6",
    "gen-m9/abc2939ebd1e2365",
    "gen-m9/b4173fe25109a238",
    "gen-m9/b6593d7dd13f2dac",
    "gen-m9/b8ee55934c5b824c",
    "gen-m9/bbdfc06cae0e0d71",
    "gen-m9/bc40d37cb4d8301d",
    "gen-m9/bcb24385baafb2f0",
    "gen-m9/c18dbf21f11093f3",
    "gen-m9/c2974c061384fdd9",
    "gen-m9/c468f5046aadfbbc",
    "gen-m9/c6cfda6bd10cae93",
    "gen-m9/caa1eed48b1883fe",
    "gen-m9/cde551ea787f713a",
    "gen-m9/d8266bc15533702a",
    "gen-m9/e6cf3b20a014e6a3",
    "gen-m9/eced4bb6a0234c9e",
    "gen-m9/ee2497a5e683c5d9",
    "gen-m9/f2bd5f380ec002bf",
    "gen-m9/f34f66fb7f81e673",
    "gen-m9/fd9fa22d212080d6",
    "gen-m9/fec39efaba9284c3",
    "probes/aggregate-overflow",
    "probes/ftest-decomposition",
    "probes/legacy-value-argument-intersection-b",
    "probes/multiarea-more",
    "probes/stat-argument-modes",
    "probes/stat-order-floor-batch",
    "probes/t2576-error-type-cell-paired",
    "probes/t2579-array-value-edges",
]
_FRESH = ["probes-numeric/t1749-devsq-fresh"]


def _numbers(args: tuple[Arg, ...]) -> list[float]:
    """The numbers DEVSQ averages. A range gives only its numbers; a single value is read as a
    literal argument (TRUE is 1, numeric text its number), and other text as a one-cell
    reference, which skips it. The sweep cannot tell a one-cell reference from a literal, so a
    one-cell reference to a bool or numeric text is a miss for every candidate alike."""
    out: list[float] = []
    for a in args:
        if isinstance(a, list):
            out += [v for v in a if isinstance(v, float) and not isinstance(v, bool)]
        elif isinstance(a, bool):
            out.append(float(a))
        elif isinstance(a, float):
            out.append(a)
        elif isinstance(a, str):
            try:
                out.append(float(a))
            except ValueError:
                continue  # a one-cell reference to text (a literal would be #VALUE!)
    if not out:
        msg = "DEVSQ of no numbers is #NUM!"
        raise ValueError(msg)
    return out


def _mean(values: list[float]) -> float:
    total = 0.0
    for v in values:
        total += v
    return total / len(values)


def two_pass(*args: Arg) -> float:
    """Σ(x - mean)², summed left to right around the double mean (the engine since T-1749)."""
    values = _numbers(args)
    mean = _mean(values)
    total = 0.0
    for v in values:
        total += (v - mean) * (v - mean)
    return total


def one_pass(*args: Arg) -> float:
    """Σx² - (Σx)²/n, each sum left to right (the engine before T-1749)."""
    values = _numbers(args)
    s = 0.0
    sq = 0.0
    for v in values:
        s += v
        sq += v * v
    return sq - s * s / len(values)


def correctly_rounded(*args: Arg) -> float:
    """Σ(x - mean)² of the double inputs around their exact mean, rounded once."""
    values = [Fraction(v) for v in _numbers(args)]
    mean = sum(values, Fraction(0)) / len(values)
    return float(sum(((v - mean) ** 2 for v in values), Fraction(0)))


def _candidates() -> list[Candidate]:
    return [
        candidate(REFERENCE, correctly_rounded, "reference"),
        candidate("two-pass around the double mean", two_pass, "two-pass"),
        candidate("one-pass sum of squares less (sum)^2/n", one_pass, "one-pass"),
    ]


_PATTERN = r"^=DEVSQ\((.*)\)$"

FITS = [
    Fit(
        name="devsq",
        description="DEVSQ summation form over the gen-m9 and probes cells it owns (T-1749)",
        cases=_OWNED,
        pattern=_PATTERN,
        candidates=_candidates,
    ),
    Fit(
        name="devsq-fresh",
        description="DEVSQ summation form on the sweep filled after the form was chosen (T-1749, R4.2)",
        cases=_FRESH,
        pattern=_PATTERN,
        candidates=_candidates,
    ),
]
