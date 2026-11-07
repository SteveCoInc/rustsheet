"""T-4465: score a filled t4465 case against the candidate rules: python3 -I docs/research/t4465/score.py <set>.

Each column's cell is predicted from the extent n a rule reads: SEQUENCE rows/cols and RANDARRAY give n rows (or
columns), `#CALC!` (E14) at 0 and `#VALUE!` (E3) below 0; TAKE gives n rows, E14 at 0 (|n| for a negative n);
DROP 100003-|n| rows; EXPAND n rows, E3 below 1; WRAPROWS ceil(3/w) x w, WRAPCOLS w x ceil(3/w) with w = min(n, 3), E6 below 1;
MAKEARRAY E3 for any fractional extent. The frozen rule reads `shared` for B, C, D and `trunc` for F:J.
"""

import csv
import json
import math
import sys


def predict(col: str, n: int, x: float) -> str:
    if col in "BCD":
        if n == 0:
            return "E14"
        if n < 0:
            return "E3"
        return f"1x{n}" if col == "C" else f"{n}x1"
    if col == "E":
        return "E3" if x != math.floor(x) else f"{n}x1"
    if col == "F":
        return "E14" if n == 0 else f"{abs(n)}x1"
    if col == "G":
        return f"{100003 - abs(n)}x1"
    if col == "H":
        return "E3" if n < 1 else f"{n}x1"
    if n < 1:
        return "E6"
    # A wrap of 3 or more holds the whole 3-cell vector: 1x3 (WRAPROWS) or 3x1 (WRAPCOLS), not padded out to the
    # wrap (main I29, J29), so only wraps below 3 show the extent read.
    wrap = min(n, 3)
    other = math.ceil(3 / wrap)
    return f"{other}x{wrap}" if col == "I" else f"{wrap}x{other}"


def main(which: str) -> None:
    base = f"eval/suites/probes/cases/t4465-extent-near-integer-{which}"
    oracle = json.load(open(f"{base}.oracle.json"))["sheets"]["P"]
    case = json.load(open(f"{base}.case.json"))["sheets"][0]["cells"]
    rows = list(csv.DictReader(open(f"docs/research/t4465/predictions-{which}.tsv"), delimiter="\t"))
    frozen = {c: ("shared" if c in "BCD" else "trunc") for c in "BCDEFGHIJ"}
    for rule in ("frozen", "shared", "eps", "f32", "floor", "trunc"):
        hit = total = 0
        for row in rows:
            r = row["row"]
            x = float(eval(case[f"A{r}"][1:].replace("^", "**")))  # the generator's own arithmetic text
            for col in "BCDEFGHIJ":
                cell = oracle.get(f"{col}{r}")
                if cell is None:
                    continue
                n = int(float(row[frozen[col] if rule == "frozen" else rule]))
                total += 1
                hit += predict(col, n, x) == str(cell["v"])
        print(f"{which}\t{rule}\t{hit}/{total}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "main")
