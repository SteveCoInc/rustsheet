"""Discriminating-probe designer for IRR: inputs where two IRR models disagree (T-4837).

Phase 0, item 3 of docs/research/irr-2026-11-07/plan.md, with T-3698's method: probe cells are chosen because two
rules disagree on them, so each oracle cell separates the two. Given two `irr_driver` candidates (names of
`irr_driver.CANDIDATES`), the designer
1. draws random inputs over the plan's section 3 axes (`draw`): the number of flows, the sign pattern, the absolute
   scale, the dynamic range between the outlay and the other flows, the root's location (through the ratio of the
   outlay to the inflows), the guess, and the structure (level annuities, zero runs, trailing zeros);
2. hill-climbs on the flows (`climb`) from the draws that came closest without disagreeing, and walks the
   neighbourhood of every disagreeing input for more of them;
3. writes a spec case with one `=IRR({flows},guess)` cell per input and a predictions file holding each model's
   value and terminal event per cell, committed before any fill (as docs/research/t4027/predictions.md was).

Two outcomes disagree (`disagree`) when their types differ (a number against an error, or two error codes) or when
both are numbers whose 15-significant-digit renderings differ. Every flow and guess is drawn with at most 15
significant digits, so the formula literal reads back as the same double: Excel truncates a literal's digits past
the 15th (docs/excel-semantics/numbers.md, "15-digit literal parsing").

  cd tools && uv run python -m rustsheet_tools.fitlab.irr_discriminate \\
      --a "base, cap 200" --b "rate: hold on flat + run on past d = 1, cap 200" \\
      --count 40 --seed 1 --out ../eval/suites/probes/cases/t4838-irr-base-vs-rate

writes `<out>.case.json` and `<out>.predictions.json`. Filling the case is T-4838's.
"""

from __future__ import annotations

import argparse
import json
import math
import random
from dataclasses import dataclass
from pathlib import Path

from rustsheet_tools.fitlab.kernels.irr_driver import CANDIDATES, Options, Outcome, irr

DIGITS = 15  # the significant digits a formula literal keeps (numbers.md, "15-digit literal parsing")
MAX_FLOWS = 120  # long enough for the 21+-flow residual (plan.md section 3, axis 1), short enough to run fast
CLIMB_STEPS = 40  # hill-climb steps per start
NEIGHBOURS = 3  # neighbours tried around each disagreeing input


@dataclass(frozen=True)
class Probe:
    """One input: the flows and the guess, as IRR's two arguments."""

    flows: tuple[float, ...]
    guess: float

    def formula(self) -> str:
        return (
            "=IRR({" + ",".join(literal_text(c) for c in self.flows) + "}," + literal_text(self.guess) + ")"
        )


def literal(x: float) -> float:
    """`x` with at most 15 significant digits, so its literal reads back as this double."""
    if x == 0 or not math.isfinite(x):
        return x
    return float(f"{x:.{DIGITS}g}")


def literal_text(x: float) -> str:
    """`x` as a formula literal of at most 15 significant digits: the shortest round-trip form (`repr`), never the
    full binary expansion of a large integer, which Excel would cut to another double."""
    text = repr(float(x)).replace("e+", "E+").replace("e-", "E-")
    return text.removesuffix(".0")


def _signature(value: float | str) -> str:
    """The outcome as compared: an error code, or a number's 15-significant-digit rendering."""
    return value if isinstance(value, str) else f"{value:.{DIGITS}g}"


def disagree(a: Outcome, b: Outcome) -> bool:
    """Whether two outcomes differ in type or past the 15th digit."""
    return _signature(a.value) != _signature(b.value)


def distance(a: Outcome, b: Outcome) -> float:
    """How far apart two outcomes are: infinite when they disagree in type, else the relative difference of the two
    numbers (0 for two equal errors). The hill-climb raises it until `disagree` holds."""
    if isinstance(a.value, str) or isinstance(b.value, str):
        return 0.0 if a.value == b.value else math.inf
    scale = max(abs(a.value), abs(b.value))
    return 0.0 if scale == 0 else abs(a.value - b.value) / scale


def draw(rng: random.Random) -> Probe:
    """A random input over plan.md section 3's axes."""
    n = rng.choice([2, 3, 4, 6, 10, 20, 21, 25, 40, 60, MAX_FLOWS])
    scale = 10.0 ** rng.uniform(-15, 15)
    # Root location: the outlay against one inflow sets it, from near -1 (outlay far above the inflows' sum) through
    # 0 and 0.1 to far roots (an outlay tiny against the inflows).
    ratio = 10.0 ** rng.uniform(-8, 3) * (n - 1)
    structure = rng.choice(["level", "noisy", "zero-run", "trailing-zeros", "growing"])
    inflows = []
    for i in range(n - 1):
        c = 1.0
        if structure == "noisy":
            c = rng.uniform(0.2, 2.0)
        elif structure == "growing":
            c = 1.05**i
        elif structure == "zero-run" and 0 < i < (n - 1) // 2:
            c = 0.0
        inflows.append(c)
    if structure == "trailing-zeros":
        cut = max(1, (n - 1) // 2)
        inflows = inflows[:cut] + [0.0] * (len(inflows) - cut)
    flows = [-ratio, *inflows]
    pattern = rng.choice(["conventional", "conventional", "middle", "positive-first"])
    if pattern == "middle" and n > 3:
        flows[rng.randrange(2, n)] *= -rng.uniform(0.5, 3.0)
    elif pattern == "positive-first":
        flows = [-c for c in flows]
    # Dynamic range: sometimes one flow dwarfs the others (a 1E+15 outlay against flows of 1).
    if rng.random() < 0.2:
        flows[rng.randrange(n)] *= 10.0 ** rng.uniform(5, 15)
    guess = rng.choice([0.1, 0.1, 0.0, rng.uniform(-0.99, 1.0), 10.0 ** rng.uniform(-3, 4), -0.5])
    return Probe(tuple(literal(c * scale) for c in flows), literal(guess))


def perturb(rng: random.Random, probe: Probe) -> Probe:
    """A neighbour: one flow, or the guess, moved by a random relative step."""
    flows = list(probe.flows)
    if rng.random() < 0.2:
        return Probe(probe.flows, literal(probe.guess * (1 + rng.gauss(0, 0.1)) + rng.gauss(0, 1e-3)))
    i = rng.randrange(len(flows))
    flows[i] = literal(
        flows[i] * (1 + rng.gauss(0, 0.05)) if flows[i] else rng.gauss(0, 1) * max(map(abs, flows))
    )
    return Probe(tuple(flows), probe.guess)


@dataclass
class Found:
    """A disagreeing input and both models' outcomes on it."""

    probe: Probe
    a: Outcome
    b: Outcome


def _run(probe: Probe, opts: Options) -> Outcome:
    return irr(list(probe.flows), probe.guess, opts)


def climb(rng: random.Random, start: Probe, a: Options, b: Options, steps: int = CLIMB_STEPS) -> Found | None:
    """Hill-climb on the flows from `start` until the models disagree: keep a neighbour whose `distance` is not
    smaller. The first disagreeing input, or None."""
    probe, best = start, distance(_run(start, a), _run(start, b))
    for _ in range(steps):
        candidate = perturb(rng, probe)
        oa, ob = _run(candidate, a), _run(candidate, b)
        if disagree(oa, ob):
            return Found(candidate, oa, ob)
        d = distance(oa, ob)
        if d >= best:
            probe, best = candidate, d
    return None


def search(a_name: str, b_name: str, count: int, seed: int, draws: int = 400) -> list[Found]:
    """Up to `count` distinct inputs on which candidates `a_name` and `b_name` disagree: random draws, then the
    neighbourhoods of the disagreeing ones, then hill-climbs from the closest of the rest."""
    a, b = CANDIDATES[a_name][0], CANDIDATES[b_name][0]
    rng = random.Random(seed)
    found: dict[Probe, Found] = {}
    near: list[tuple[float, Probe]] = []

    def add(item: Found) -> None:
        if len(found) < count:
            found.setdefault(item.probe, item)

    for _ in range(draws):
        if len(found) >= count:
            break
        probe = draw(rng)
        oa, ob = _run(probe, a), _run(probe, b)
        if disagree(oa, ob):
            add(Found(probe, oa, ob))
        else:
            near.append((distance(oa, ob), probe))
    for item in list(found.values()):
        for _ in range(NEIGHBOURS):
            neighbour = perturb(rng, item.probe)
            oa, ob = _run(neighbour, a), _run(neighbour, b)
            if disagree(oa, ob):
                add(Found(neighbour, oa, ob))
    near.sort(key=lambda t: -t[0])
    for _, probe in near:
        if len(found) >= count:
            break
        if (climbed := climb(rng, probe, a, b)) is not None:
            add(climbed)
    return list(found.values())


def _prediction(outcome: Outcome) -> dict[str, float | str]:
    return {"value": outcome.value, "event": outcome.event}


def write(out: Path, a_name: str, b_name: str, found: list[Found], seed: int) -> None:
    """`<out>.case.json`, a pending spec case with one IRR cell per input in column A, and `<out>.predictions.json`
    with each model's value and terminal event per cell."""
    cells = {f"A{i}": f.probe.formula() for i, f in enumerate(found, 1)}
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"IRR inputs where two fit-lab models disagree in type or past the 15th digit (T-4837 designer, seed {seed}):"
            f" '{a_name}' against '{b_name}'. Each model's prediction per cell is in the .predictions.json beside this"
            " case, written before the fill."
        ),
        "tags": ["M9", "probe"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
    }
    predictions = {
        "models": [a_name, b_name],
        "seed": seed,
        "cells": {
            f"P!A{i}": {a_name: _prediction(f.a), b_name: _prediction(f.b)} for i, f in enumerate(found, 1)
        },
    }
    Path(f"{out}.case.json").write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")
    Path(f"{out}.predictions.json").write_text(json.dumps(predictions, indent=1) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument("--a", required=True, choices=sorted(CANDIDATES), help="first irr_driver candidate")
    parser.add_argument("--b", required=True, choices=sorted(CANDIDATES), help="second irr_driver candidate")
    parser.add_argument("--count", type=int, default=40, help="disagreeing inputs to find")
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--draws", type=int, default=400, help="random draws before the hill-climbs")
    parser.add_argument(
        "--out", type=Path, required=True, help="path stem of the case and the predictions file"
    )
    args = parser.parse_args(argv)
    found = search(args.a, args.b, args.count, args.seed, args.draws)
    write(args.out, args.a, args.b, found, args.seed)
    print(f"{len(found)} disagreeing inputs -> {args.out}.case.json, {args.out}.predictions.json")


if __name__ == "__main__":
    main()
