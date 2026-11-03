"""Decode probes/t3584-match-fold-weightless's oracle sidecar (T-3584).

Prints the BMP weightless set (Calc!R1:R256 digit 9: MATCH and `=` both equal U+0524) as code-point
ranges, the code points with other masks (MATCH wildcards), and the per-block MATCH equivalence
classes (Calc!T) that are neither a plain one-to-one case pair nor all weightless.
Run from the repository root: `python3 docs/research/t3584/decode.py > docs/research/t3584/weightless-bmp.txt`.
"""

import json
from pathlib import Path

CASES = Path("eval/suites/probes/cases")
CASE = "t3584-match-fold-weightless"


def main() -> None:
    oracle = json.loads((CASES / f"{CASE}.oracle.json").read_text())["sheets"]["Calc"]
    cells = json.loads((CASES / f"{CASE}.case.json").read_text())["sheets"][0]["cells"]
    mask = "".join(oracle[f"R{p + 1}"]["v"] for p in range(256))
    weightless = [cp for cp, m in enumerate(mask) if m == "9"]
    ranges: list[tuple[int, int]] = []
    for cp in weightless:
        if ranges and ranges[-1][1] == cp - 1:
            ranges[-1] = (ranges[-1][0], cp)
        else:
            ranges.append((cp, cp))
    print(f"# {len(weightless)} weightless BMP code points in {len(ranges)} ranges")
    for lo, hi in ranges:
        print(f"{lo:04X}" if lo == hi else f"{lo:04X}-{hi:04X}")
    errors = [cp for cp, m in enumerate(mask) if m == "e"]
    print(f"# UNICHAR errors (e): {len(errors)} code points")
    others = " ".join(f"{cp:04X}={m}" for cp, m in enumerate(mask) if m not in "09e")
    print(f"# other masks: {others}")
    print("# MATCH classes beyond one-to-one case pairs (not all weightless)")
    seen: set[int] = set()
    row = 1
    while f"U{row}" in cells:
        cp = int(cells[f"U{row}"][2:], 16)
        text = oracle[f"T{row}"]["v"]
        members = sorted(int(x, 16) for x in text.split(",")) if text else []
        row += 1
        if cp in seen or len(members) < 2 or all(mask[m] == "9" for m in members):
            continue
        seen.update(members)
        ch = chr(cp)
        lower, upper = ch.lower(), ch.upper()
        simple = {
            cp,
            ord(lower) if len(lower) == 1 else cp,
            ord(upper) if len(upper) == 1 else cp,
        }
        if len(members) == 2 and set(members) <= simple:
            continue
        print(" ".join(f"{m:04X}" for m in members))


if __name__ == "__main__":
    main()
