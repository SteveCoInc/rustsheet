"""T-3953: score t3953-f-tail-bfrac-tiny per column (A F.DIST.RT, B FDIST, C BETA.DIST sheet v, D BETA.DIST literal
v) and split (tune / hold): the engine against Excel, Excel and the engine against the 50-digit math sidecar, and
Excel's relative error against the math (quantiles). Optionally compares a second fill (decision 0112).

Run: cd tools && uv run python -I ../docs/research/t3953/score.py <repo> [<second .oracle.json>]"""

from __future__ import annotations

import json
import sys
from decimal import Decimal, getcontext
from pathlib import Path

from rustsheet_tools.fitlab.score import ulp_distance, within_fifteen_digits
from rustsheet_tools.snapshot import show_cells

getcontext().prec = 60
REPO = Path(sys.argv[1])
NAME = "t3953-f-tail-bfrac-tiny"
CASES = REPO / "eval" / "suites" / "probes-numeric" / "cases"
case = json.loads((CASES / f"{NAME}.case.json").read_text())
oracle = json.loads((CASES / f"{NAME}.oracle.json").read_text())["sheets"]["Calc"]
math_cells = json.loads((CASES / f"{NAME}.math.json").read_text())["sheets"]["Calc"]
shown = show_cells(f"probes-numeric/{NAME}.case.json")


def num(v: object) -> float | None:
    return v if isinstance(v, float) else (float(v) if isinstance(v, int) else None)


def rel(x: float, ref: Decimal) -> float:
    """Signed relative error of x against ref."""
    return float((Decimal(x) - ref) / ref) if ref else float("inf")


if len(sys.argv) > 2:
    second = json.loads(Path(sys.argv[2]).read_text())["sheets"]["Calc"]
    diff = sorted(k for k in oracle if second.get(k) != oracle[k])
    print(f"second fill: {len(diff)} of {len(oracle)} cells differ: {' '.join(diff) or 'none'}")

tot: dict[str, dict[str, int]] = {}
errs: dict[str, dict[str, list[float]]] = {"Excel": {}, "engine": {}}
worst: list[tuple[float, str, str]] = []
for key, hyp in case["hypotheses"].items():
    cell = key.split("!")[1]
    split, kind = hyp.split("/")[1], hyp.split("/")[-1]
    ex = num(oracle.get(cell, {}).get("v"))
    try:
        act = float(shown[key]["actual"])
    except (KeyError, ValueError):
        act = None
    # The math tool defines no F.DIST.RT / FDIST, so A and B take row's C value: the same exact quantity,
    # I_v(d2/2, TRUNC(d1)/2) at v = d2/(TRUNC(d1)*X + d2).
    m = math_cells.get(cell, {}).get("v") or math_cells.get("C" + cell[1:], {}).get("v")
    c_ex = num(oracle.get("C" + cell[1:], {}).get("v"))
    for group in (f"{kind}/{split}", f"{kind}/all"):
        t = tot.setdefault(group, {})

        def bump(k: str, ok: bool, t: dict[str, int] = t) -> None:
            t[k] = t.get(k, 0) + int(ok)

        bump("cells", True)
        bump("excel numeric", ex is not None)
        if ex is None or act is None:
            continue
        bump("engine=excel exact", act == ex)
        bump("engine~excel <=4ulp", ulp_distance(act, ex) <= 4)
        bump("engine~excel 15d", within_fifteen_digits(ex, act))
        if kind in ("fdistrt", "fdist", "beta-literal-v") and c_ex is not None:
            bump("excel = excel's C (sheet-v BETA.DIST) exact", ex == c_ex)
        if m is not None:
            ref = Decimal(m)
            cr = float(ref)
            bump("excel=math(cr) exact", ex == cr)
            bump("excel~math <=4ulp", ulp_distance(ex, cr) <= 4)
            bump("excel~math 15d", within_fifteen_digits(cr, ex))
            bump("engine=math(cr) exact", act == cr)
            bump("engine~math <=4ulp", ulp_distance(act, cr) <= 4)
            bump("engine~math 15d", within_fifteen_digits(cr, act))
            if group.endswith("/all"):
                e = rel(ex, ref)
                errs["Excel"].setdefault(kind, []).append(e)
                errs["engine"].setdefault(kind, []).append(rel(act, ref))
                worst.append((abs(e), cell, hyp))

print(json.dumps(tot, indent=1))
for who, by_kind in errs.items():
    for kind, signed in by_kind.items():
        es = sorted(abs(e) for e in signed)
        q = {p: es[min(len(es) - 1, int(p * len(es)))] for p in (0.5, 0.9, 0.99)}
        print(
            f"{kind}: {who} |rel. error| vs math: median {q[0.5]:.2g}, p90 {q[0.9]:.2g}, p99 {q[0.99]:.2g}, "
            f"max {es[-1]:.2g}; high {sum(e > 0 for e in signed)}, low {sum(e < 0 for e in signed)}; "
            f"> 1E-14: {sum(e > 1e-14 for e in es)}, > 1E-12: {sum(e > 1e-12 for e in es)}, > 1E-4: {sum(e > 1e-4 for e in es)}"
        )
for e, cell, hyp in sorted(worst, reverse=True)[:8]:
    print(f"worst {cell} {e:.3g} {hyp}")
