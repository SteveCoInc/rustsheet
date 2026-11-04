"""T-3576: where NEGBINOM.DIST's cdf (Excel's incomplete beta K(s, f + 1, p, 1 - p)) is #NUM!.

For each cell of probes/t3576-negbinom-cdf-huge-s, follow TOMS 708 bratio's dispatch for a, b > 1 (the line-by-line
model is docs/research/t2464/r2-bratio/bratio_excel.py `kernel`) and, on the L140 branch, print the natural log of
the largest partial sum of bup's terms d_i = prod_{l < i} (apb + l) / (ap1 + l) * x (attempt 1, which does not
separate the probe) and the ln of the running ratio product R = prod_{l < n-1} (apb + l) / (ap1 + l) alone (attempt 2,
which does: Excel is #NUM! exactly where R passes the largest double, ln 709.78). Excel's value is printed beside it.
Pass another case's path (without .case.json) to score it, e.g. eval/suites/probes-numeric/cases/t3553-negbinom-huge-s. Run from the repository root: python3 docs/research/t3576/bup_partial_sum.py
"""

import json
import math
import re
import sys

CASE = "eval/suites/probes/cases/t3576-negbinom-cdf-huge-s"


def main() -> None:
    case = sys.argv[1] if len(sys.argv) > 1 else CASE
    with open(case + ".case.json") as f:
        cells = json.load(f)["sheets"][0]["cells"]
    with open(case + ".oracle.json") as f:
        excel = json.load(f)["sheets"]["Calc"]
    rows = []
    for key in sorted(cells, key=lambda k: int(k[1:])):
        formula = cells[key]
        m = re.match(r"=NEGBINOM\.DIST\(([^,]+),([^,]+),([^,]+),TRUE\)", formula)
        if m:
            fails, succ, p = (float(v.replace("2^31", "2147483648")) for v in m.groups())
            a, b, x = succ, fails + 1, p
        else:
            m = re.match(r"=BETA\.DIST\(([^,]+),([^,]+),([^,]+),TRUE\)", formula)
            if not m:
                rows.append((None, None, key, formula, excel[key]["v"], "not a cdf"))
                continue
            x, a, b = (float(v) for v in m.groups())
        y = 1 - x
        lam = (a + b) * y - b if a > b else a - (a + b) * x
        if lam < 0:
            a, b, x, y = b, a, y, x
        if b >= 40 or b * x <= 0.7:
            rows.append((None, None, key, formula, excel[key]["v"], "not L140"))
            continue
        n = int(b)
        b -= n
        if b == 0:
            n -= 1
            b = 1.0
        # L140: bup(b0, a0, y0, x0, n), then bpser (x0 <= 0.7) or bgrat (x0 > 0.7).
        apb, ap1 = b + a, b + 1
        log_d, top, log_r = 0.0, 0.0, 0.0
        for l in range(n - 1):
            log_d += math.log((apb + l) / (ap1 + l) * y)
            log_r += math.log((apb + l) / (ap1 + l))
            top = max(top, log_d)
        rows.append((log_r, top, key, formula, excel[key]["v"], "bup+bgrat" if x > 0.7 else "bup+bpser"))
    print("cell  ln R     ln sum   branch    Excel                formula")
    for log_r, top, key, formula, value, branch in sorted(rows, key=lambda r: -1 if r[0] is None else r[0]):
        r_shown = "" if log_r is None else f"{log_r:8.2f}"
        shown = "" if top is None else f"{top:8.2f}"
        print(f"{key:4} {r_shown:>8} {shown:>8} {branch:9} {value!s:20} {formula}")


if __name__ == "__main__":
    main()
