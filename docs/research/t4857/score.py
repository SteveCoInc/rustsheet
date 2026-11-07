"""T-4857: score Excel, the engine and the 50-digit math on the F family's equivalence-class probes, per class and
per pair, find the regime changes along the 1-D sweeps and edges, and join the prior-evidence counts.

    cd tools && uv run python ../docs/research/t4857/score.py BEFORE.json [CELLS.json] > ../docs/research/t4857/TABLES.md

BEFORE.json is `audit.py --exclude t4857`'s output (the Excel cells on file before this task). The engine's values
come from `rustsheet-eval show --all` (target/release/rustsheet-eval), the math from each case's `.math.json`
sidecar (`rustsheet-accuracy math`: mpmath at 50+ digits on the exact ratio d1 x/(d1 x + d2)).
"""

from __future__ import annotations

import json
import math
import sys
from collections import defaultdict
from decimal import Decimal
from pathlib import Path

import audit as A
import cells as C
import fclasses as F
from rustsheet_tools.fitlab.score import ulp_distance, within_fifteen_digits
from rustsheet_tools.snapshot import show_cells

CASES = ("probes-numeric/t4857-f-classes", "probes/t4857-f-kinds")
THIN = 3


def within_digits(expected: float, actual: float, digits: int) -> bool:
    """`within_fifteen_digits` at `digits` significant digits (13 is decision 0267's family bar)."""
    if digits == 15:
        return within_fifteen_digits(expected, actual)
    if not (math.isfinite(expected) and math.isfinite(actual)):
        return False
    if expected == 0.0:
        return actual == 0.0
    exponent = Decimal(repr(abs(expected))).adjusted()
    return abs(actual - expected) <= float(f"5e{exponent - digits}")


def rel(a: float, m: Decimal) -> float:
    """|a - m| / |m| (0 for an exact 0 against 0)."""
    if m == 0:
        return 0.0 if a == 0 else math.inf
    return float(abs(Decimal(repr(a)) - m) / abs(m))


def math_double(m: Decimal) -> float:
    """The math value rounded to a double, a subnormal flushed to 0 as Excel does."""
    v = float(m)
    return 0.0 if abs(v) < F.DBL_MIN else v


def parse_actual(text: str | None) -> tuple[str, float | None]:
    """('num', value), (error code, None) or (other, None) from `show`'s printed value."""
    if text is None:
        return "missing", None
    t = text.strip()
    if t.startswith("#"):
        return t, None
    if t in ("TRUE", "FALSE"):
        return "bool", None
    try:
        return "num", float(t)
    except ValueError:
        return "text", None


class Stat:
    """Agreement counts for one class or pair."""

    def __init__(self) -> None:
        self.n = 0
        self.types = 0
        self.c = defaultdict(int)
        self.worst: dict[str, tuple[float, str]] = {}

    def add(self, key: str, ok: bool) -> None:
        self.c[key] += ok

    def worse(self, key: str, err: float, cell: str) -> None:
        if err > self.worst.get(key, (-1.0, ""))[0]:
            self.worst[key] = (err, cell)


def score_cells() -> list[dict]:
    rows = []
    for case_id in CASES:
        suite, name = case_id.split("/")
        path = C.ROOT / "eval/suites" / suite / "cases" / f"{name}.case.json"
        doc = json.loads(path.read_text(encoding="utf-8"))
        hyp = doc.get("hypotheses", {})
        shown = show_cells(f"{case_id}.case.json")
        for cell in C.read_cells([(suite, path)]):
            key = f"{cell.sheet}!{cell.a1}"
            label = C.oracle_label(cell.oracle)
            xl = C.oracle_number(cell.oracle)
            g_label, g = parse_actual(shown.get(key, {}).get("actual"))
            if cell.oracle.get("t") == "b":
                xl_text = str(cell.oracle.get("v")).upper()
                g_label = (
                    "bool" if shown.get(key, {}).get("actual", "").upper() in ("TRUE", "FALSE") else g_label
                )
                same = shown.get(key, {}).get("actual", "").upper() == xl_text
            else:
                same = None
            call = cell.calls[0] if cell.calls else None
            classes = {}
            if call and call.numbers and call.top_level and len(call.numbers) >= 3:
                classes = A.numeric_classes(call.function, call.numbers, POSITIONS)
            rows.append(
                {
                    "case": case_id,
                    "cell": key,
                    "tag": hyp.get(key, ""),
                    "formula": cell.formula,
                    "function": call.function if call else "",
                    "numbers": call.numbers if call else None,
                    "top": bool(call and call.top_level),
                    "xl_label": label,
                    "xl": xl,
                    "xl_bool": cell.oracle.get("v") if label == "bool" else None,
                    "g_label": g_label,
                    "g": g,
                    "bool_same": same,
                    "math": Decimal(cell.math) if cell.math not in (None, "") else None,
                    "classes": classes,
                }
            )
    return rows


POSITIONS: dict = {}


def fill_positions() -> None:
    for case_id in CASES:
        suite, name = case_id.split("/")
        path = C.ROOT / "eval/suites" / suite / "cases" / f"{name}.case.json"
        for cell in C.read_cells([(suite, path)]):
            for call in cell.calls:
                if call.numbers and call.function not in A.INV + A.TEST and len(call.numbers) >= 3:
                    x, r1, r2 = call.numbers[:3]
                    d1, d2 = float(math.trunc(r1)), float(math.trunc(r2))
                    if (
                        1 <= d1 <= F.F_DF_MAX
                        and 1 <= d2 <= F.F_DF_MAX
                        and x >= F.DBL_MIN
                        and math.isfinite(d1 * x)
                    ):
                        if (x, d1, d2) not in POSITIONS:
                            POSITIONS[(x, d1, d2)] = A._position((x, d1, d2))


def measure(r: dict) -> dict:
    """Per-cell verdicts: type agreement, then digits for numbers."""
    out = {"type_ok": r["xl_label"] == r["g_label"] and (r["bool_same"] is not False)}
    xl, g, m = r["xl"], r["g"], r["math"]
    if r["xl_label"] == "num" and r["g_label"] == "num":
        out["ge15"] = within_digits(xl, g, 15)
        out["ge13"] = within_digits(xl, g, 13)
        out["ge_exact"] = xl == g
        out["ge_ulp"] = ulp_distance(xl, g)
        out["ge_rel"] = abs(g - xl) / abs(xl) if xl else (0.0 if g == 0 else math.inf)
    if m is not None:
        md = math_double(m)
        if r["xl_label"] == "num":
            out["em15"] = within_digits(md, xl, 15)
            out["em13"] = within_digits(md, xl, 13)
            out["em_rel"] = rel(xl, m) if md != 0 or xl != 0 else 0.0
        if r["g_label"] == "num":
            out["gm15"] = within_digits(md, g, 15)
            out["gm13"] = within_digits(md, g, 13)
            out["gm_rel"] = rel(g, m) if md != 0 or g != 0 else 0.0
    return out


def fmt(x: float) -> str:
    if x is None or x < 0:
        return "-"
    if x == 0:
        return "0"
    if math.isinf(x):
        return "inf"
    return f"{x:.1e}"


def table(
    stats: dict[str, Stat], prior: dict[str, int] | None, title: str, only_interesting: bool = False
) -> list[str]:
    out = [f"### {title}", ""]
    out.append(
        "| Class | Prior Excel cells | New cells | Type agree | Excel~math 15/13 | Engine~Excel 15/13 | Engine~math 15/13 |"
        " Worst Excel-math | Worst engine-Excel | Worst engine-math |"
    )
    out.append("|---|---|---|---|---|---|---|---|---|---|")
    for k in sorted(stats):
        s = stats[k]
        p = prior.get(k, 0) if prior is not None else "-"
        nnum = s.c["ge_n"]
        bad13 = nnum - s.c["ge13"]
        if only_interesting and not (bad13 or s.types < s.n):
            continue
        out.append(
            f"| {k} | {p} | {s.n} | {s.types}/{s.n} | {s.c['em15']}/{s.c['em13']} of {s.c['em_n']} | "
            f"{s.c['ge15']}/{s.c['ge13']} of {nnum} | {s.c['gm15']}/{s.c['gm13']} of {s.c['gm_n']} | "
            f"{fmt(s.worst.get('em', (-1, ''))[0])} {s.worst.get('em', (0, ''))[1]} | "
            f"{fmt(s.worst.get('ge', (-1, ''))[0])} {s.worst.get('ge', (0, ''))[1]} | "
            f"{fmt(s.worst.get('gm', (-1, ''))[0])} {s.worst.get('gm', (0, ''))[1]} |"
        )
    out.append("")
    return out


def accumulate(s: Stat, r: dict, v: dict) -> None:
    s.n += 1
    s.types += v["type_ok"]
    a1 = r["cell"].split("!")[1]
    for key in ("ge", "em", "gm"):
        if f"{key}13" in v:
            s.c[f"{key}_n"] += 1
            s.add(f"{key}15", v[f"{key}15"])
            s.add(f"{key}13", v[f"{key}13"])
            s.worse(key, v[f"{key}_rel"], a1)


SERIES_KEYS = ("sweep-", "edge-x/", "edge-df/", "edge-p/", "zero/")


def series_of(tag: str) -> str | None:
    if not tag.startswith(SERIES_KEYS):
        return None
    parts = tag.split("/")
    if tag.startswith("edge-x/"):
        return "/".join(parts[:4])
    if tag.startswith("edge-df/"):
        return "/".join(parts[:5])
    return "/".join(parts[:3]) if tag.startswith(("edge-p/", "zero/")) else tag


def regime_changes(rows: list[dict], verdicts: list[dict]) -> list[str]:
    """Along each 1-D series (sorted by the argument that varies), report where Excel's result changes type, where
    Excel's error against the math jumps by 100x or more between neighbours (a loss or gain of two digits), and where
    the engine's agreement with Excel flips."""
    series: dict[str, list[tuple[float, dict, dict]]] = defaultdict(list)
    for r, v in zip(rows, verdicts, strict=True):
        name = series_of(r["tag"])
        if name is None or not r["numbers"] or r["cell"].split("!")[1][0] != "A":
            continue
        nums = r["numbers"]
        if r["tag"].startswith(("sweep-df1", "edge-df/")):
            key = nums[1] if "pos2" not in r["tag"] else nums[2]
            if r["tag"].startswith("edge-df/") and "pos2" not in r["tag"]:
                key = nums[1]
        elif r["tag"].startswith("sweep-df2"):
            key = nums[2]
        else:
            key = nums[0]
        series[name].append((key, r, v))
    out = []
    for name in sorted(series):
        pts = sorted(series[name], key=lambda t: t[0])
        prev = None
        for key, r, v in pts:
            a1 = r["cell"].split("!")[1]
            err = v.get("em_rel")
            if prev is not None:
                pkey, pr, pv, perr = prev
                notes = []
                if pr["xl_label"] != r["xl_label"]:
                    notes.append(f"Excel {pr['xl_label']} -> {r['xl_label']}")
                if err is not None and perr is not None:
                    lo, hi = sorted((max(err, 1e-18), max(perr, 1e-18)))
                    if hi / lo >= 100 and hi > 1e-14:
                        notes.append(f"Excel-math rel err {fmt(perr)} -> {fmt(err)}")
                if pv.get("ge13") is not None and v.get("ge13") is not None and pv["ge13"] != v["ge13"]:
                    notes.append(f"engine~Excel 13 digits {pv['ge13']} -> {v['ge13']}")
                if pr["g_label"] != r["g_label"] and pr["xl_label"] == r["xl_label"]:
                    notes.append(f"engine {pr['g_label']} -> {r['g_label']}")
                if notes:
                    out.append(
                        f"| {name} | {pkey!r} -> {key!r} | {pr['cell'].split('!')[1]} -> {a1} | {'; '.join(notes)} |"
                    )
            prev = (key, r, v, err)
    return out


def main() -> None:
    before = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    fill_positions()
    rows = score_cells()
    verdicts = [measure(r) for r in rows]
    single: dict[str, dict[str, Stat]] = defaultdict(lambda: defaultdict(Stat))
    pairs: dict[str, dict[str, Stat]] = defaultdict(lambda: defaultdict(Stat))
    sections: dict[str, Stat] = defaultdict(Stat)
    functions: dict[str, Stat] = defaultdict(Stat)
    mismatches = []
    for r, v in zip(rows, verdicts, strict=True):
        sec = (
            r["tag"].split("/")[0] if not r["tag"].startswith("kind/") else "/".join(r["tag"].split("/")[:2])
        )
        accumulate(sections[sec], r, v)
        accumulate(functions[r["function"] or "other"], r, v)
        if not v["type_ok"]:
            mismatches.append(
                f"| {r['case'].split('/')[1]} {r['cell'].split('!')[1]} | `{r['formula']}` | {r['tag']} | "
                f"{r['xl_label'] if r['xl_label'] != 'bool' else r['xl_bool']} | {r['g_label']} |"
            )
        group = next((g for g, fs in A.GROUPS.items() if r["function"] in fs), None)
        if group is None or not r["classes"]:
            continue
        cls = r["classes"]
        for dim, c in cls.items():
            accumulate(single[group][f"{dim}={c}"], r, v)
        for left, right in A.PAIRS[group]:
            if left in cls and right in cls:
                accumulate(pairs[group][f"{left}x{right}={cls[left]}|{cls[right]}"], r, v)
    out = ["# T-4857 scoring tables (generated by score.py)", ""]
    out += table(sections, None, "Per section")
    out += table(functions, None, "Per function")
    for group in A.GROUPS:
        prior = {}
        for dim in ("xmag", "xpos", "df1", "df2", "kernel", "density", "path", "p", "centre"):
            for c, n in before["single"].get(f"{group}|{dim}", {}).items():
                prior[f"{dim}={c}"] = n
        out += table(single[group], prior, f"{group}: per class")
        pprior = {}
        for left, right in A.PAIRS[group]:
            for k, n in before["pairs"].get(f"{group}|{left}x{right}", {}).items():
                pprior[f"{left}x{right}={k}"] = n
        out += table(
            pairs[group],
            pprior,
            f"{group}: pairs where the engine is off Excel at 13 digits or a type differs",
            only_interesting=True,
        )
        # Overfitting findings: pairs with little prior evidence where the engine misses Excel at 13 digits.
        thin = [
            (k, s)
            for k, s in pairs[group].items()
            if pprior.get(k, 0) < THIN and (s.c["ge_n"] - s.c["ge13"] > 0 or s.types < s.n)
        ]
        out.append(
            f"#### {group}: low-prior-evidence pairs (< {THIN} Excel cells before) where the engine drifts"
        )
        out.append("")
        for k, s in sorted(thin):
            out.append(
                f"- `{k}`: prior {pprior.get(k, 0)}, new {s.n}, engine~Excel 13 digits {s.c['ge13']}/{s.c['ge_n']}, "
                f"types {s.types}/{s.n}, worst engine-Excel {fmt(s.worst.get('ge', (-1, ''))[0])} at "
                f"{s.worst.get('ge', (0, ''))[1]}"
            )
        out.append("")
    out += [
        "## Type and error mismatches (Excel against the engine)",
        "",
        "| Cell | Formula | Tag | Excel | Engine |",
        "|---|---|---|---|---|",
    ]
    out += mismatches
    out += [
        "",
        "## Regime changes along the 1-D series",
        "",
        "| Series | Argument | Cells | Change |",
        "|---|---|---|---|",
    ]
    out += regime_changes(rows, verdicts)
    dump = [
        {k: (str(v) if isinstance(v, Decimal) else v) for k, v in r.items() if k != "math"}
        | {"math": str(r["math"]) if r["math"] is not None else None}
        | {f"v_{k}": val for k, val in vd.items()}
        for r, vd in zip(rows, verdicts, strict=True)
    ]
    if len(sys.argv) > 2:
        Path(sys.argv[2]).write_text(json.dumps(dump, indent=0, default=str) + "\n", encoding="utf-8")
    print("\n".join(out))


if __name__ == "__main__":
    main()
