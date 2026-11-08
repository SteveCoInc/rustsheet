"""Score elapsed-count models on every elapsed-count cell with fraction digits: python3 count_models.py [CASE ...].

Reads every `=TEXT(<serial expr>,"[h|m|s]...")` cell with `.0`, `.00` or `.000` of the t4323, t4332, t4333, t4370,
t4389 and t4466 probe cases (or only the cases named), and prints each model's count with a deterministic 30% hold-out
by cell hash (sha256 of "case!cell", last digit-mod-10 below 3), as decision 0188 R4.1 asks.
`x` is the serial, `s` the fraction scale, `u` the unit in seconds, `y = x + 0.5/(86400*s)`.

- rounded: the exact value rounded half up to the shown digit (the correctly rounded candidate);
- register: `floor(y*86400/u)`, the engine before T-4333;
- chain60: `floor(((y*60)*1440)/u)`, the same y multiplied by 60 then by 1440 in doubles;
- max: the larger of the two, frozen 2026-11-08 before probes/t4466-count-fresh was drawn (a structural model, no fitted constant; see README).
"""

import glob
import hashlib
import json
import math
import os
import re
import sys
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(HERE, "../../../eval/suites/probes/cases/")
# probes/t4466-count-fresh is the fresh sweep drawn after the freeze: score_fresh.py scores it, never this split.
PATTERN = re.compile(r"t43(23|32|33|70|89)|t4466-count-hypotheses$")
TEXT = re.compile(r'^=TEXT\((.*),"([^"]+)"\)$')
COUNT = re.compile(r"^\[(h|m|s)\][^ ]*?\.(0+)")


def serial(expr):
    """The serial a probe formula's arithmetic gives: Python doubles evaluate it as Excel does (left to right, `^`
    exact for the powers of two used)."""
    return eval(expr.replace("^", "**"), {"__builtins__": {}})  # noqa: S307 (probe formulas we generated)


def rows(names):
    out = []
    for path in sorted(glob.glob(CASES + "t4*.case.json")):
        name = os.path.basename(path)[: -len(".case.json")]
        if names and name not in names or not names and not PATTERN.match(name):
            continue
        oracle_path = CASES + name + ".oracle.json"
        if not os.path.exists(oracle_path):
            continue
        cells = json.load(open(path))["sheets"][0]["cells"]
        oracle = json.load(open(oracle_path))["sheets"]["Sheet1"]
        for cell, formula in cells.items():
            m = TEXT.match(formula) if isinstance(formula, str) else None
            f = m and COUNT.match(m.group(2))
            if not f or cell not in oracle:
                continue
            u = {"h": 3600, "m": 60, "s": 1}[f.group(1)]
            count = int(re.match(r"\d+", oracle[cell]["v"]).group())
            out.append((name, cell, serial(m.group(1)), u, 10 ** len(f.group(2)), count))
    return out


def register(x, u, s):
    return math.floor((x + 0.5 / (86400 * s)) * 86400 / u)


def chain60(x, u, s):
    return math.floor(((x + 0.5 / (86400 * s)) * 60) * 1440 / u)


def rounded(x, u, s):
    """The correctly rounded candidate: the exact serial's seconds rounded half up to the shown digit, truncated."""
    return math.floor(math.floor(Fraction(x) * 86400 * s + Fraction(1, 2)) / s / u)


MODELS = {
    "rounded": rounded,
    "register": register,
    "chain60": chain60,
    "max": lambda x, u, s: max(register(x, u, s), chain60(x, u, s)),
}


def split(name, cell):
    return "test" if int(hashlib.sha256(f"{name}!{cell}".encode()).hexdigest(), 16) % 10 < 3 else "train"


if __name__ == "__main__":
    data = rows(set(sys.argv[1:]))
    for model, fn in MODELS.items():
        tally = {"train": [0, 0], "test": [0, 0]}
        misses = []
        for name, cell, x, u, s, count in data:
            t = tally[split(name, cell)]
            t[1] += 1
            if fn(x, u, s) == count:
                t[0] += 1
            else:
                misses.append(f"{name}!{cell}")
        print(
            f"{model:9} train {tally['train'][0]}/{tally['train'][1]}  test {tally['test'][0]}/{tally['test'][1]}  "
            f"misses {len(misses)}: {' '.join(misses[:40])}"
        )
