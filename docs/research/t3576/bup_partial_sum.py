"""T-3576 attempt 1: where NEGBINOM.DIST's cdf (Excel's incomplete beta K(s, f + 1, p, 1 - p)) is #NUM!.

For each cell of probes/t3576-negbinom-cdf-huge-s, follow TOMS 708 bratio's dispatch for a, b > 1 (the line-by-line
model is docs/research/t2464/r2-bratio/bratio_excel.py `kernel`) and, on the L140 branch, print the natural log of
the largest partial sum of bup's terms d_i = prod_{l < i} (apb + l) / (ap1 + l) * x. Excel's value is printed beside
it. Run from the repository root: python3 docs/research/t3576/bup_partial_sum.py
"""

import json
import math
import re

CASE = "eval/suites/probes/cases/t3576-negbinom-cdf-huge-s"


def main() -> None:
    with open(CASE + ".case.json") as f:
        cells = json.load(f)["sheets"][0]["cells"]
    with open(CASE + ".oracle.json") as f:
        excel = json.load(f)["sheets"]["Calc"]
    rows = []
    for key in sorted(cells, key=lambda k: int(k[1:])):
        formula = cells[key]
        m = re.match(r"=NEGBINOM\.DIST\(([^,]+),([^,]+),([^,]+),TRUE\)", formula)
        if m:
            fails, succ, p = (float(v) for v in m.groups())
            a, b, x = succ, fails + 1, p
        else:
            m = re.match(r"=BETA\.DIST\(([^,]+),([^,]+),([^,]+),TRUE\)", formula)
            assert m, formula
            x, a, b = (float(v) for v in m.groups())
        y = 1 - x
        lam = (a + b) * y - b if a > b else a - (a + b) * x
        if lam < 0:
            a, b, x, y = b, a, y, x
        if b >= 40 or b * x <= 0.7:
            rows.append((None, key, formula, excel[key]["v"], "not L140"))
            continue
        n = int(b)
        b -= n
        if b == 0:
            n -= 1
            b = 1.0
        # L140: bup(b0, a0, y0, x0, n), then bpser (x0 <= 0.7) or bgrat (x0 > 0.7).
        apb, ap1 = b + a, b + 1
        log_d, top = 0.0, 0.0
        for l in range(n - 1):
            log_d += math.log((apb + l) / (ap1 + l) * y)
            top = max(top, log_d)
        rows.append((top, key, formula, excel[key]["v"], "bup+bgrat" if x > 0.7 else "bup+bpser"))
    for top, key, formula, value, branch in sorted(rows, key=lambda r: -1 if r[0] is None else r[0]):
        shown = "" if top is None else f"{top:8.2f}"
        print(f"{key:4} {shown:>8} {branch:9} {value!s:20} {formula}")


if __name__ == "__main__":
    main()
