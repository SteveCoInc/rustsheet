"""T-4435's grid: F tails and BETA.DIST where the incomplete beta has one huge shape outside large_shape_beta's
bounds, against 60-digit mpmath (docs/research/t4434/mathref.py).

Run from the repository root with the tools venv:

    tools/.venv/bin/python -I docs/research/t4435/grid.py make            # writes grid-references.json (slow, ~10 min)
    tools/.venv/bin/python -I docs/research/t4435/grid.py case DIR        # a scratch eval root for the engine
    target/release/rustsheet-eval show --all --root DIR t4435/grid.case.json > show.txt
    tools/.venv/bin/python -I docs/research/t4435/grid.py score show.txt  # the engine against the math

Seed 4435 (random.Random), drawn before any kernel was scored, and disjoint from T-4434's case (seed 4434). Integer
degrees of freedom; every x rounded to 12 significant digits and every non-integer shape to 6, so each literal is read
as the double it names (Excel and the engine read formula literals at 15 digits). 800 points:

- r1-rt  200 F.DIST.RT, df1 log-uniform in [3, 79], df2 in [1E4, 2E6]: a0 = df2/2 of 5E3 to 1E6 at b0 = df1/2 < 40.
         The upper tail is log-uniform in [1E-200, 0.5] (x solved for it).
- r1-cdf 100 F.DIST(..,TRUE) at the same ranges, the lower tail log-uniform in [1E-200, 0.5].
- r2-rt  200 F.DIST.RT, df1 in [80, 400], df2 in [1E5, 1E10]: b0 = 40 to 200 at a0 of 5E4 and up. x = r times the
         mean, r log-uniform in [0.7, 3].
- r2-cdf 100 F.DIST(..,TRUE) there, r log-uniform in [0.5, 1.5].
- b1-a   50 BETA.DIST(x,a,b,TRUE), a in [5E3, 1E6], b in [1.5, 40] (region 1, huge first shape): 1 - x = r b/a,
         r log-uniform in [0.3, 3].
- b1-b   50 the same with the shapes' roles swapped (a in [1.5, 40], b huge): x = r a/b.
- b2-a   50 BETA.DIST, a in [5E4, 5E9], b in [40, 200] (region 2, T-4214's orientation): 1 - x = r b/a.
- b2-b   50 a in [40, 200], b in [5E4, 5E9]: x = r a/b.
The shapes are log-uniform. The F math is at the exact ratio d1 x/(d2 + d1 x), BETA.DIST's at the double x.
"""

from __future__ import annotations

import json
import math
import random
import re
import statistics
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "docs/research/t4434"))
import gen_probe  # noqa: E402
import mathref  # noqa: E402

mp.mp.dps = 60
SEED = 4435
REFS = HERE / "grid-references.json"
GROUPS = ("r1-rt", "r1-cdf", "r2-rt", "r2-cdf", "b1-a", "b1-b", "b2-a", "b2-b")
round_sig, log_uniform, lit = gen_probe.round_sig, gen_probe.log_uniform, gen_probe.lit


def _log_lower(x: float, d1: int, d2: int) -> float:
    m = mp.mpf(d1) * mp.mpf(x)
    with mp.workdps(260):
        p = mathref._at(mp.mpf(d1) / 2, mp.mpf(d2) / 2, m, m + d2, False, 240)
        return float(mp.log10(p)) if p > mp.mpf(10) ** -225 else -1000.0


def solve_lower(target: float, d1: int, d2: int) -> float:
    """x where log10 F.DIST(x, d1, d2, TRUE) = target (the cdf rises with x)."""
    lo, hi = math.log(1e-9), math.log(1.0)
    for _ in range(50):
        mid = (lo + hi) / 2
        if _log_lower(math.exp(mid), d1, d2) < target:
            lo = mid
        else:
            hi = mid
    return round_sig(math.exp((lo + hi) / 2), 12)


def points() -> list[tuple[str, str, float, float, float]]:
    """(group, function, x, p, q): F calls take (x, df1, df2), BETA.DIST (x, a, b)."""
    rng = random.Random(SEED)
    out: list[tuple[str, str, float, float, float]] = []
    pending_rt, pending_cdf = [], []
    for _ in range(200):
        pending_rt.append(
            (
                round(log_uniform(rng, 3, 79)),
                round(log_uniform(rng, 1e4, 2e6)),
                rng.uniform(-200, math.log10(0.5)),
            )
        )
    for _ in range(100):
        pending_cdf.append(
            (
                round(log_uniform(rng, 3, 79)),
                round(log_uniform(rng, 1e4, 2e6)),
                rng.uniform(-200, math.log10(0.5)),
            )
        )
    for _ in range(200):
        d1, d2 = round(log_uniform(rng, 80, 400)), round(log_uniform(rng, 1e5, 1e10))
        out.append(("r2-rt", "F.DIST.RT", round_sig(log_uniform(rng, 0.7, 3.0) * d2 / (d2 - 2), 12), d1, d2))
    for _ in range(100):
        d1, d2 = round(log_uniform(rng, 80, 400)), round(log_uniform(rng, 1e5, 1e10))
        out.append(("r2-cdf", "F.DIST", round_sig(log_uniform(rng, 0.5, 1.5) * d2 / (d2 - 2), 12), d1, d2))
    for group, (alo, ahi), (blo, bhi), huge_first in (
        ("b1-a", (5e3, 1e6), (1.5, 40), True),
        ("b1-b", (1.5, 40), (5e3, 1e6), False),
        ("b2-a", (5e4, 5e9), (40, 200), True),
        ("b2-b", (40, 200), (5e4, 5e9), False),
    ):
        for _ in range(50):
            a, b = round_sig(log_uniform(rng, alo, ahi), 6), round_sig(log_uniform(rng, blo, bhi), 6)
            r = log_uniform(rng, 0.3, 3.0)
            x = round_sig(1.0 - r * b / a, 12) if huge_first else round_sig(r * a / b, 12)
            out.append((group, "BETA.DIST", x, a, b))
    # The tail searches come last so the draws above never depend on them.
    rt = [("r1-rt", "F.DIST.RT", gen_probe.solve_x(t, d1, d2), d1, d2) for d1, d2, t in pending_rt]
    cdf = [("r1-cdf", "F.DIST", solve_lower(t, d1, d2), d1, d2) for d1, d2, t in pending_cdf]
    return rt + cdf + out


def math_value(fn: str, x: float, p: float, q: float) -> mp.mpf:
    if fn == "F.DIST.RT":
        return mathref.f_upper(x, int(p), int(q))
    if fn == "F.DIST":
        return mathref.f_lower(x, int(p), int(q))
    if x > 0.5:  # a huge first shape: I_x(a, b) = 1 - I_{1-x}(b, a), with 1 - x exact
        return mathref.ibeta(mp.mpf(q), mp.mpf(p), 1 - mp.mpf(x), 1, upper=True)
    return mathref.beta_lower(x, p, q)


def formula(fn: str, x: float, p: float, q: float) -> str:
    if fn == "BETA.DIST":
        return f"=BETA.DIST({lit(x)},{lit(p)},{lit(q)},TRUE)"
    tail = ",TRUE" if fn == "F.DIST" else ""
    return f"={fn}({lit(x)},{int(p)},{int(q)}{tail})"


def make() -> None:
    rows = []
    for group, fn, x, p, q in points():
        rows.append(
            [group, formula(fn, x, p, q), mp.nstr(math_value(fn, x, p, q), 40, min_fixed=1, max_fixed=0)]
        )
    REFS.write_text("[\n" + ",\n".join(json.dumps(r) for r in rows) + "\n]\n")
    print(f"wrote {REFS.relative_to(ROOT)}: {len(rows)} points")


def load() -> list[tuple[str, str, mp.mpf]]:
    return [(g, f, mp.mpf(v)) for g, f, v in json.loads(REFS.read_text())]


def case(root: Path) -> None:
    """A scratch eval root: suite t4435 with one case, whose oracle is the math rounded to a double."""
    rows = load()
    suite = root / "eval/suites/t4435"
    suite.mkdir(parents=True, exist_ok=True)
    (suite / "suite.toml").write_text(
        'name = "t4435"\ndescription = "T-4435 grid (scratch)"\nkind = "spec"\nroot = "."\n'
        'include = ["*.case.json"]\nlicense = "Apache-2.0"\norigin = "generated"\ngate = false\n'
        'baseline = "cells"\nmilestone = ["M9"]\n'
    )
    cells = {f"A{i}": f for i, (_, f, _) in enumerate(rows, start=1)}
    spec = {
        "format": "rustsheet-case/v1",
        "description": "T-4435 grid",
        "tags": ["M9"],
        "expected": {},
        "check": [f"Calc!{k}" for k in cells],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    (suite / "grid.case.json").write_text(json.dumps(spec))
    oracle = {
        "format": "rustsheet-oracle/v1",
        "source": {"kind": "excel-graph", "application": "math (scratch)", "input_sha256": "0"},
        "sheets": {"Calc": {f"A{i}": {"t": "n", "v": float(v)} for i, (_, _, v) in enumerate(rows, start=1)}},
    }
    (suite / "grid.oracle.json").write_text(json.dumps(oracle))
    print(f"wrote {suite}")


def engine(show: Path) -> dict[int, float]:
    out, row = {}, None
    for line in show.read_text().splitlines():
        if line.startswith("Calc!A"):
            row = int(line.split()[0][len("Calc!A") :])
        elif row is not None and line.strip().startswith("actual"):
            text = line.split(":", 1)[1].strip()
            out[row] = float(text) if re.fullmatch(r"[-+0-9.eE]+|inf|NaN", text) else math.inf
    return out


def score(show: Path) -> None:
    rows, got = load(), engine(show)
    print("| group | points | <=1E-15 | <=1E-14 | <=1E-13 | median rel | worst rel | worst point |")
    print("|---|---:|---:|---:|---:|---:|---:|---|")
    for group in GROUPS + ("all",):
        errs = []
        for i, (g, f, v) in enumerate(rows, start=1):
            if group in (g, "all"):
                e = float(abs(mp.mpf(got[i]) - v) / abs(v)) if math.isfinite(got[i]) else math.inf
                errs.append((e, f))
        es = sorted(e for e, _ in errs)
        worst = max(errs)
        print(
            f"| {group} | {len(es)} | {sum(e <= 1e-15 for e in es)} | {sum(e <= 1e-14 for e in es)} | "
            f"{sum(e <= 1e-13 for e in es)} | {statistics.median(es):.1e} | {worst[0]:.1e} | `{worst[1]}` |"
        )


def compare(before: Path, after: Path) -> None:
    """Every point whose error grew by more than 1.5 times, and the counts that moved."""
    rows, b, a = load(), engine(before), engine(after)
    worse = 0
    for i, (g, f, v) in enumerate(rows, start=1):
        eb = float(abs(mp.mpf(b[i]) - v) / abs(v))
        ea = float(abs(mp.mpf(a[i]) - v) / abs(v))
        if ea > 1.5 * eb and ea > 2e-16:
            worse += 1
            print(f"worse {g} {f}: {eb:.2e} -> {ea:.2e}")
    print(f"{worse} points worse by more than 1.5 times (and above 2E-16)")


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "make":
        make()
    elif cmd == "case":
        case(Path(sys.argv[2]))
    elif cmd == "score":
        score(Path(sys.argv[2]))
    elif cmd == "compare":
        compare(Path(sys.argv[2]), Path(sys.argv[3]))
