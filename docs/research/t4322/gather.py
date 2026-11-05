"""T-4322: every oracle cell of a large-shape incomplete gamma tail with literal arguments.

Collects GAMMA.DIST/GAMMADIST(x,a,b,TRUE), CHISQ.DIST(x,df,TRUE), CHISQ.DIST.RT/CHIDIST(x,df) and
POISSON.DIST/POISSON(k,m,TRUE) cells of every eval suite whose Excel value is the smaller tail
(P below the mode, Q above it), as (case, ref, formula, a, y, tail, value).
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
NUM = r"\s*(-?[0-9.]+(?:E[-+]?[0-9]+)?)\s*"
PATS = [
    (re.compile(rf"^=(?:_xlfn\.)?(GAMMA\.DIST|GAMMADIST)\({NUM},{NUM},{NUM},(TRUE|1)\)$", re.I), "gamma"),
    (re.compile(rf"^=(?:_xlfn\.)?(CHISQ\.DIST)\({NUM},{NUM},(TRUE|1)\)$", re.I), "chisq"),
    (re.compile(rf"^=(?:_xlfn\.)?(CHISQ\.DIST\.RT|CHIDIST)\({NUM},{NUM}\)$", re.I), "chirt"),
    (re.compile(rf"^=(?:_xlfn\.)?(POISSON\.DIST|POISSON)\({NUM},{NUM},(TRUE|1)\)$", re.I), "pois"),
]


def gather(min_shape=20.0):
    out = []
    for case in sorted(ROOT.glob("eval/suites/*/cases/*.case.json")):
        orc_path = case.with_name(case.name.replace(".case.json", ".oracle.json"))
        if not orc_path.exists():
            continue
        try:
            sheets = json.loads(case.read_text())["sheets"]
            orc = json.loads(orc_path.read_text())["sheets"]
        except (KeyError, ValueError):
            continue
        for sh in sheets:
            o = orc.get(sh.get("name", "Calc"), {})
            for ref, f in sh.get("cells", {}).items():
                if not isinstance(f, str):
                    continue
                for pat, kind in PATS:
                    m = pat.match(f.replace(" ", ""))
                    if not m:
                        continue
                    cell = o.get(ref)
                    if not cell or cell.get("t") != "n":
                        break
                    v = cell["v"]
                    args = [float(t) for t in m.groups()[1:4] if t is not None and t.upper() != "TRUE"]
                    if kind == "gamma":
                        x, a, b = args[:3]
                        y, tail = x / b, "P"
                    elif kind == "chisq":
                        x, df = args[:2]
                        a, y, tail = df / 2, x / 2, "P"
                    elif kind == "chirt":
                        x, df = args[:2]
                        a, y, tail = df / 2, x / 2, "Q"
                    else:
                        k, mean = args[:2]
                        if k != int(k) or k < 0:
                            break
                        a, y, tail = k + 1, mean, "Q"
                    if a < min_shape or not (y > 0) or v <= 0:
                        break
                    smaller = "P" if y < a else "Q"
                    if tail == smaller and v < 0.5:
                        out.append((case.name.split(".")[0], ref, f, a, y, tail, v))
                    break
    return out


if __name__ == "__main__":
    rows = gather()
    print(len(rows))
    win = [r for r in rows if 2 / 3 <= r[4] / r[3] <= 1.5]
    print("in window", len(win))
