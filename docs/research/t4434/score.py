"""T-4434: score Excel and the engine against 60-digit math on probes-numeric/t4434-huge-shape-outside-large-shape-beta.

Run from the repository root (paths are relative to it) with the tools venv, after a release build:

    target/release/rustsheet-eval show --all probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json > show.txt
    tools/.venv/bin/python -I docs/research/t4434/score.py show.txt [--refresh]

The math (mathref.py) is taken at the double arguments, and for the F calls at the exact ratio d1 x / (d2 + d1 x),
not the rounded double the kernels form. It is cached in references.json (40 significant digits); `--refresh`
recomputes it. Excel's values are the committed oracle sidecar.

Prints, per region and per decade of the huge shape (a0 = df2/2 for F, b for BETA.DIST), the counts within 1E-15,
1E-14 and 1E-12 relative for each side and each side's worst error, then Excel's per-region ulp profile (how near
correctly rounded it is), then every cell where either side is more than 1E-13 off (the owner lists of the task).
"""

from __future__ import annotations

import json
import math
import re
import statistics
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import gen_probe  # noqa: E402
import mathref  # noqa: E402

CASE = ROOT / "eval/suites/probes-numeric/cases" / f"{gen_probe.STEM}.case.json"
REFS = HERE / "references.json"
CALL = re.compile(r"=(F\.DIST\.RT|F\.DIST|BETA\.DIST)\(([^,]+),([^,]+),([^,)]+)(?:,TRUE)?\)")
mp.mp.dps = 60
REGION = {"A": "region 1 F.DIST.RT", "B": "region 2 F.DIST.RT", "C": "region 2 F.DIST", "D": "BETA.DIST"}


def calls() -> dict[str, tuple[str, float, float, float]]:
    cells = json.loads(CASE.read_text())["sheets"][0]["cells"]
    out = {}
    for ref, formula in cells.items():
        m = CALL.fullmatch(formula)
        assert m, formula
        out[ref] = (m[1], float(m[2]), float(m[3]), float(m[4]))
    return out


def math_values(fn: str, x: float, p: float, q: float) -> tuple[mp.mpf, mp.mpf]:
    """(the math at the exact F ratio, the math at the double ratio a kernel forms); one value twice for BETA.DIST."""
    if fn == "F.DIST.RT":
        return mathref.f_upper(x, int(p), int(q)), mathref.f_upper_rounded(x, int(p), int(q))
    if fn == "F.DIST":
        return mathref.f_lower(x, int(p), int(q)), mathref.f_lower_rounded(x, int(p), int(q))
    v = mathref.beta_lower(x, p, q)
    return v, v


def references(refresh: bool) -> tuple[dict[str, mp.mpf], dict[str, mp.mpf]]:
    """({cell: math at the exact ratio}, {cell: math at the rounded ratio}), cached in references.json."""
    if REFS.exists() and not refresh:
        raw = json.loads(REFS.read_text())
    else:
        raw = {}
        for ref, (fn, x, p, q) in calls().items():
            raw[ref] = [mp.nstr(v, 40, min_fixed=1, max_fixed=0) for v in math_values(fn, x, p, q)]
        order = sorted(raw, key=lambda r: (r[0], int(r[1:])))
        REFS.write_text("{\n" + ",\n".join(f'"{r}": {json.dumps(raw[r])}' for r in order) + "\n}\n")
    return {k: mp.mpf(v[0]) for k, v in raw.items()}, {k: mp.mpf(v[1]) for k, v in raw.items()}


def excel_values() -> dict[str, float | str]:
    data = json.loads(CASE.with_name(f"{gen_probe.STEM}.oracle.json").read_text())["sheets"]["Calc"]
    return {ref: (v["v"] if v.get("t") == "n" else f"#{v.get('v')}") for ref, v in data.items()}


def engine_values(show: Path) -> dict[str, float | str]:
    out: dict[str, float | str] = {}
    ref = None
    for line in show.read_text().splitlines():
        if line.startswith("Calc!"):
            ref = line.split()[0][len("Calc!") :]
        elif ref and line.strip().startswith("actual"):
            text = line.split(":", 1)[1].strip()
            try:
                out[ref] = float(text)
            except ValueError:
                out[ref] = text
    return out


def rel(v: float | str, m: mp.mpf) -> float:
    if isinstance(v, str):
        return math.inf
    return float(abs(mp.mpf(v) - m) / abs(m))


def ulps(v: float | str, m: mp.mpf) -> float:
    if isinstance(v, str):
        return math.inf
    return float(abs(mp.mpf(v) - m) / mp.mpf(math.ulp(float(m))))


def conditioned(v: float | str, m: mp.mpf) -> float:
    """The relative error in units of max(1, |ln m|) * 2^-53. A double kernel ends in exp(-L) with L = |ln m|, and L
    itself is rounded to an ulp of L, so even a perfect double kernel is about L ulp off at a deep tail. A side whose
    value stays within a few of these units is as accurate as double arithmetic allows there."""
    return rel(v, m) / (max(1.0, abs(float(mp.log(m)))) * 2.0**-53)


def within15(expected: float | str, actual: float | str) -> bool:
    """`closeness::within_fifteen_digits`: within half a unit of the 15th significant digit of `expected`."""
    if isinstance(expected, str) or isinstance(actual, str) or not math.isfinite(actual):
        return expected == actual
    if expected == 0:
        return actual == 0
    exponent = math.floor(math.log10(abs(expected)))
    return abs(actual - expected) <= float(f"5e{exponent - 15}")


def b0_at_least_40(call: tuple[str, float, float, float]) -> bool:
    """Whether bratio's b0 is 40 or more, so the cell takes bfrac (or basym) rather than L140 bup+bgrat. bratio forms
    lambda = (a + b) y - b for a > b, else a - (a + b) x, and swaps (a0, b0) = (b, a) where it is negative. K(a, b, x)
    is (df1/2, df2/2, d1 x/(d1 x + d2)) for the F calls, (a, b, x) for BETA.DIST."""
    fn, x, p, q = call
    if fn == "BETA.DIST":
        a, b, u = p, q, x
    else:
        a, b, u = p / 2, q / 2, p * x / (p * x + q)
    lam = (a + b) * (1 - u) - b if a > b else a - (a + b) * u
    return (a if lam < 0 else b) >= 40


def owner(call: tuple[str, float, float, float], excel: float | str, m: mp.mpf) -> str:
    """Who owns a cell where the engine is not within 15 digits of Excel: T-4435 where the math, correctly rounded, is
    (an accurate kernel then matches Excel); else Excel is the side off the math, T-4214 at b0 >= 40 (its bfrac
    arithmetic) and T-3209 below (its bgrat digits).

    This amends T-4434's criterion, which moved a cell away from T-4435 only where Excel is more than 1E-13 off the
    math. T-4435 is the `accuracy` stage: it can bring a cell within 15 digits of Excel only where the math is. A cell
    where Excel is 1E-15 to 1E-13 off and the math is not within 15 digits of Excel is Excel's own digits, which only
    a model of Excel's arithmetic reaches (T-4214, T-3209). main() prints how far off Excel is per owner."""
    if not isinstance(excel, str) and within15(excel, float(m)):
        return "T-4435"
    return "T-4214" if b0_at_least_40(call) else "T-3209"


def spans(cells: list[str]) -> str:
    """Cells as single cells and colon spans of consecutive rows in one column (A1:A3, B7), in the given order."""
    out: list[str] = []
    start = prev = cells[0]
    for ref in cells[1:] + [""]:
        if ref and ref[0] == prev[0] and int(ref[1:]) == int(prev[1:]) + 1:
            prev = ref
            continue
        out.append(start if start == prev else f"{start}:{prev}")
        start = prev = ref
    return ", ".join(out)


def huge_shape(call: tuple[str, float, float, float]) -> float:
    """The huge shape: df2/2 for the F calls, b for BETA.DIST."""
    return call[3] / 2 if call[0] != "BETA.DIST" else call[3]


def row(errs: list[float]) -> str:
    n = len(errs)
    counts = "/".join(f"{sum(e <= t for e in errs):3}" for t in (1e-15, 1e-14, 1e-12))
    return f"{counts} of {n:3}, worst {max(errs):.1e}"


def main() -> None:
    show = Path(sys.argv[1])
    refs, at_double = references("--refresh" in sys.argv)
    cs, xl, en = calls(), excel_values(), engine_values(show)
    groups: dict[tuple[str, int], list[str]] = {}
    for ref, call in cs.items():
        decade = math.floor(math.log10(huge_shape(call)))
        groups.setdefault((ref[0], decade), []).append(ref)

    print(
        "| region | huge shape | cells | Excel <=1E-15/1E-14/1E-12, worst | engine <=1E-15/1E-14/1E-12, worst |"
    )
    print("|---|---|---:|---|---|")
    for col in "ABCD":
        keys = sorted(k for k in groups if k[0] == col) + [(col, None)]
        for key in keys:
            refs_here = groups[key] if key[1] is not None else [r for r in cs if r[0] == col]
            band = f"1E{key[1]} to 1E{key[1] + 1}" if key[1] is not None else "**all**"
            ex = [rel(xl[r], refs[r]) for r in refs_here]
            eg = [rel(en[r], refs[r]) for r in refs_here]
            print(f"| {REGION[col]} | {band} | {len(refs_here)} | {row(ex)} | {row(eg)} |")

    print("\nUlp error against the math at the double F ratio fl(d1 x / (d1 x + d2)), the kernel's own error")
    print("(correctly rounded = 0.5 ulp or less; BETA.DIST has no ratio, so it is the same math)")
    print("| region | side | <=0.5 ulp | <=1 ulp | <=4 ulp | median ulp | 90th pct ulp | worst ulp |")
    print("|---|---|---:|---:|---:|---:|---:|---:|")
    for col in "ABCD":
        refs_here = [r for r in cs if r[0] == col]
        for side, vals in (("Excel", xl), ("engine", en)):
            u = sorted(ulps(vals[r], at_double[r]) for r in refs_here)
            print(
                f"| {REGION[col]} | {side} | {sum(e <= 0.5 for e in u)} | {sum(e <= 1 for e in u)} | "
                f"{sum(e <= 4 for e in u)} | {statistics.median(u):.1f} | {u[int(0.9 * len(u))]:.1f} | {u[-1]:.3g} |"
            )

    print(
        "\nError in units of max(1, |ln value|) * 2^-53 (see `conditioned`), per region and decade of the huge shape"
    )
    print(
        "| region | huge shape | cells | Excel median | Excel 90th pct | Excel worst | engine median | engine worst |"
    )
    print("|---|---|---:|---:|---:|---:|---:|---:|")
    for col in "ABCD":
        for key in sorted(k for k in groups if k[0] == col):
            here = groups[key]
            ex = sorted(conditioned(xl[r], refs[r]) for r in here)
            eg = sorted(conditioned(en[r], refs[r]) for r in here)
            print(
                f"| {REGION[col]} | 1E{key[1]} to 1E{key[1] + 1} | {len(here)} | {statistics.median(ex):.3g} | "
                f"{ex[int(0.9 * len(ex))]:.3g} | {ex[-1]:.3g} | {statistics.median(eg):.3g} | {eg[-1]:.3g} |"
            )

    print("\nCells where either side is more than 1E-13 relative off the math")
    print("| cell | call | Excel rel | engine rel |")
    print("|---|---|---:|---:|")
    for ref in sorted(cs, key=lambda r: (r[0], int(r[1:]))):
        ex, eg = rel(xl[ref], refs[ref]), rel(en[ref], refs[ref])
        if max(ex, eg) > 1e-13:
            fn, x, p, q = cs[ref]
            print(f"| {ref} | {fn}({x!r},{p:g},{q:g}) | {ex:.1e} | {eg:.1e} |")

    print("\nOwners of the cells where the engine is not within 15 digits of Excel (see `owner`)")
    owned: dict[str, list[str]] = {}
    for ref in sorted(cs, key=lambda r: (r[0], int(r[1:]))):
        if not within15(xl[ref], en[ref]):
            owned.setdefault(owner(cs[ref], xl[ref], refs[ref]), []).append(ref)
    for task, cells in sorted(owned.items()):
        print(f"- {task} ({len(cells)}): Calc!{spans(cells)}")
    print("\nHow far Excel is off the math on each owner's cells (relative)")
    for task, cells in sorted(owned.items()):
        errs = [rel(xl[r], refs[r]) for r in cells]
        print(
            f"- {task}: {sum(e > 1e-13 for e in errs)} more than 1E-13 off, {sum(1e-14 < e <= 1e-13 for e in errs)} "
            f"from 1E-14 to 1E-13, {sum(e <= 1e-14 for e in errs)} within 1E-14; worst {max(errs):.2g}"
        )


if __name__ == "__main__":
    main()
