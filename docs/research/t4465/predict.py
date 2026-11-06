"""Predictions for T-4465's probe cells, committed before the excel-graph fill (decision 0188 R4).

Usage: python3 docs/research/t4465/predict.py <cases_dir> <show_dir> > predictions.tsv

For every cell it prints the argument x, the engine's value before T-4465 (from
`rustsheet-eval show --all` of each case, run with placeholder expectations so the engine values
print), and the cell value each candidate reading rule predicts:
- shared: near_integer_whole (floor; up-snap within 2^-22 + 2^-33, within half that below 0);
- eps23: floor(x + 2^-23) (sequence.rs EXTENT_EPSILON);
- f32: floor of x rounded to float32;
- floor: floor(x); trunc: x truncated toward zero.
The value a whole count n gives is the function's integer-argument behavior (error kinds from
the engine at exact integers, which the probe's offset-0 cells check).
"""

from __future__ import annotations

import json
import math
import re
import struct
import sys
from pathlib import Path

TOL = 2.0**-22 + 2.0**-33


def shared(x: float) -> float:
    tol = TOL / 2 if x < 0 else TOL
    w = math.floor(x)
    return w + (1 if x - w >= 1 - tol else 0)


def eps23(x: float) -> float:
    return math.floor(x + 2.0**-23)


def f32(x: float) -> float:
    return math.floor(struct.unpack("f", struct.pack("f", x))[0])


RULES = {"shared": shared, "eps23": eps23, "f32": f32, "floor": math.floor, "trunc": math.trunc}

N_LONG = 100001


def outcome(kind: str, n: int) -> str:
    """The cell value for a whole count n (integer-argument behavior)."""
    if kind in ("seq_rows", "seq_cols", "randarray"):
        return str(n) if n >= 1 else ("#CALC!" if n == 0 else "#VALUE!")
    if kind in ("makearray", "expand"):
        return str(n) if n >= 1 else "#VALUE!"
    if kind in ("wraprows", "wrapcols"):
        return str(n) if n >= 1 else "#NUM!"
    if kind == "take":
        return "#CALC!" if n == 0 else str(min(abs(n), N_LONG))
    if kind == "drop":
        return str(N_LONG) if n == 0 else (str(N_LONG - abs(n)) if abs(n) < N_LONG else "#CALC!")
    raise ValueError(kind)


def arg_value(text: str) -> float:
    """The argument's value under Excel's precedence. Exact: every term is an integer or a power
    of two times a small integer. Excel's unary minus binds before `^`, so `-2^-22` is (-2)^-22,
    which is +2^-22 (even power): the generator's base-0 cells `-2^-k` are positive for even k."""
    text = re.sub(r"^-2\^", "(-2)^", text)
    return float(eval(text.replace("^", "**"), {"__builtins__": {}}))  # noqa: S307 - generated input


def engine_values(show: Path) -> dict[str, str]:
    values, cell = {}, None
    for line in show.read_text().splitlines():
        m = re.match(r"^A!(\w+)\s", line)
        if m:
            cell = m.group(1)
        m = re.match(r"^\s+actual\s*:\s*(.*)$", line)
        if m and cell:
            v = m.group(1).strip()
            values[cell] = v[:-2] if v.endswith(".0") else v
    return values


def main() -> None:
    cases, shows = Path(sys.argv[1]), Path(sys.argv[2])
    print("case\tcell\tformula\tx\tengine\t" + "\t".join(RULES))
    for path in sorted(cases.glob("t4465*.case.json")):
        name = path.name.removesuffix(".case.json")
        case = json.loads(path.read_text())
        kinds = dict(part.split(": ") for part in case["description"].split("Columns: ")[1].rstrip(".").split("; "))
        engine = engine_values(shows / f"{name}.show")
        cells = case["sheets"][0]["cells"]
        for ref in sorted(cells, key=lambda r: (re.sub(r"\d", "", r), int(re.sub(r"\D", "", r)))):
            formula = cells[ref]
            kind = kinds[re.sub(r"\d", "", ref)]
            # The argument is the last top-level argument of the constructor call.
            call = formula[formula.index("(", formula.index("(") + 1) + 1 : formula.rindex(")")]
            call = call[: call.rindex(")")] if call.count(")") > call.count("(") else call
            depth, parts, cur = 0, [], ""
            for ch in call:
                if ch == "," and depth == 0:
                    parts.append(cur)
                    cur = ""
                    continue
                depth += ch == "("
                depth -= ch == ")"
                cur += ch
            parts.append(cur)
            text = {"makearray": 0, "seq_cols": 1}.get(kind, len(parts) - 1 if kind in ("take", "drop", "expand", "wraprows", "wrapcols") else 0)
            x = arg_value(parts[text])
            preds = [outcome(kind, int(rule(x))) for rule in RULES.values()]
            print(f"{name}\t{ref}\t{formula}\t{x!r}\t{engine.get(ref, '?')}\t" + "\t".join(preds))


if __name__ == "__main__":
    main()
