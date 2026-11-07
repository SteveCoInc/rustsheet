"""T-3576 decision 0188 R4.1: the L140 overflow rule against the correctly rounded value, train and hold-out.

The split is the fit lab's (tools/src/rustsheet_tools/fitlab/holdout.py, T-3360): a cell is held out when the first 8
bytes of sha256("rustsheet-fitlab-holdout/v1\\0<suite>/<case>\\0Calc!A<n>") are below 300 mod 1000. Margin: 5 points
(the fit lab's default), stated before scoring. The model's cells are the engine's (`rustsheet-eval show` lists the
cells that are not bit-exact); the correctly rounded value is the case's math sidecar rounded to a double, and a cell
without one (the sidecar's `skipped`) counts as a miss for it. Run from the repository root after
`make build`: python3 docs/research/t3576/holdout.py
"""

import hashlib
import json
import re
import subprocess

CASES = [("probes", "t3576-negbinom-cdf-huge-s"), ("probes-numeric", "t3553-negbinom-huge-s")]


def held_out(case: str, key: str) -> bool:
    digest = hashlib.sha256(f"rustsheet-fitlab-holdout/v1\0{case}\0{key}".encode()).digest()
    return int.from_bytes(digest[:8], "big") % 1000 < 300


def main() -> None:
    score = {side: {"cells": 0, "model": 0, "correctly rounded": 0} for side in ("train", "test")}
    for suite, case in CASES:
        base = f"eval/suites/{suite}/cases/{case}"
        excel = json.load(open(base + ".oracle.json"))["sheets"]["Calc"]
        math = json.load(open(base + ".math.json"))["sheets"]["Calc"]
        shown = subprocess.run(
            ["target/release/rustsheet-eval", "show", f"{suite}/{case}.case.json"],
            capture_output=True, text=True, check=True,
        ).stdout
        misses = set(re.findall(r"^Calc!(A\d+)\s", shown, re.M))
        for key, cell in excel.items():
            side = score["test" if held_out(f"{suite}/{case}", f"Calc!{key}") else "train"]
            side["cells"] += 1
            side["model"] += key not in misses
            exact = math.get(key)
            if exact is not None:
                value = float(exact["v"]) if exact["t"] == "n" else exact["v"]
                side["correctly rounded"] += value == cell["v"]
    for side, s in score.items():
        print(f"{side:5} {s['cells']:3} cells: model {s['model']} bit-exact, correctly rounded {s['correctly rounded']}")


if __name__ == "__main__":
    main()
