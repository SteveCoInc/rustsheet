"""T-3795 R4.1 hold-out on the fit lab's split (`fitlab.holdout`, T-3360): the chosen models (the
engine at the branch head) against the correctly rounded candidate, per function.

Population: every IMSQRT, IMLOG10, IMLOG2 and IMSECH cell of the eval suites whose argument is a
literal or COMPLEX(x, y) of literals and whose oracle value is text, except the fresh sweep
probes-numeric/t3795-r4-im-last-digits (R4.2 is its own guard). Both candidates answer with text:
the engine its own printed text (Excel's where the cell passes, the `actual` of `rustsheet-eval
show` where it does not), the correctly rounded one each 50-digit part rounded to a double and
printed to 15 significant digits. Scored part by part by `fitlab.score` (exact, within 4 ulp,
within 15 digits); margin `DEFAULT_MARGIN` (5 points).

Run (from tools/): uv run python ../docs/research/t3795/holdout.py ACTUALS
ACTUALS: one "<case id>\t<Sheet!A1>\t<engine text>" line per engine miss.
"""

import glob
import json
import re
import sys

from mpmath import atan2, log, mp, mpc, mpf, sech, sqrt
from rustsheet_tools.fitlab.holdout import DEFAULT_MARGIN, split, verdict
from rustsheet_tools.fitlab.score import Candidate, score
from rustsheet_tools.fitlab.sweep import Cell, parse_complex_text

mp.dps = 50
ROOT = "../eval/suites"
SWEEP = "probes-numeric/t3795-r4-im-last-digits"
actual = {}
for line in open(sys.argv[1]):
    case, key, text = line.rstrip("\n").split("\t")
    actual[(case, key)] = text


def literal_parts(t):
    """A literal's parts as Excel reads them: 15 significant digits each."""
    re_part, im_part = parse_complex_text(t.strip())
    return float(f"{re_part:.15g}"), float(f"{im_part:.15g}")


def truth(fn, x, y):
    x, y = mpf(x), mpf(y)
    if fn == "IMSQRT":
        r = sqrt(mpc(x, y))
        return r.real, r.imag
    if fn == "IMSECH":
        r = sech(mpc(x, y))
        return r.real, r.imag
    base = {"IMLOG10": log(10), "IMLOG2": log(2)}[fn]
    return log(sqrt(x * x + y * y)) / base, atan2(y, x) / base


def text15(v):
    return f"{float(v):.15g}".replace("e", "E")


pat = re.compile(r'^=(IMSQRT|IMLOG10|IMLOG2|IMSECH)\((?:COMPLEX\(([^,()]+),([^,()]+)\)|"([^"]*)")\)$')
cells, rounded_text, oracle_text = {}, {}, {}
for cf in sorted(glob.glob(ROOT + "/**/*.case.json", recursive=True)):
    try:
        case = json.load(open(cf))
        oracle = json.load(open(cf.replace(".case.json", ".oracle.json")))
    except (OSError, ValueError):
        continue
    cid = cf.split("suites/")[1].replace("/cases/", "/").removesuffix(".case.json")
    if cid == SWEEP:
        continue
    for sheet in case.get("sheets", []):
        osh = oracle.get("sheets", {}).get(sheet["name"], {})
        for address, formula in sheet.get("cells", {}).items():
            m = pat.match(formula) if isinstance(formula, str) else None
            if not m or osh.get(address, {}).get("t") != "s":
                continue
            fn = m.group(1)
            try:
                if m.group(4) is not None:
                    x, y = literal_parts(m.group(4))
                else:
                    x, y = float(m.group(2)), float(m.group(3))
                er, ei = parse_complex_text(osh[address]["v"])
                tr, ti = truth(fn, x, y)
            except (ValueError, ZeroDivisionError):
                continue
            cell = Cell(cid, sheet["name"], address, formula, (x, y), er, None, ei)
            cells.setdefault(fn, []).append(cell)
            sign = "+" if float(ti) >= 0 else "-"
            rounded_text[(cid, cell.key)] = f"{text15(tr)}{sign}{text15(abs(ti))}i"
            oracle_text[(cid, cell.key)] = osh[address]["v"]

chosen = Candidate("chosen", lambda c: actual.get((c.case, c.key), oracle_text[(c.case, c.key)]))
rounded = Candidate("correctly rounded", lambda c: rounded_text[(c.case, c.key)])
print("function  side   n     chosen exact/4ulp/15dig   correctly rounded exact/4ulp/15dig")
for fn in sorted(cells):
    train, test = split(cells[fn])
    scores = {}
    for side, part in (("train", train), ("test", test)):
        for cand in (chosen, rounded):
            s = score(part, cand)
            scores[(side, cand.name)] = s
        c, r = scores[(side, "chosen")], scores[(side, "correctly rounded")]
        print(f"{fn:8}  {side:5} {c.total:4}   {c.exact:4} {c.ulp:4} {c.digits15:4}   {r.exact:4} {r.ulp:4} {r.digits15:4}")
    v = verdict(scores[("train", "chosen")], scores[("test", "chosen")], scores[("test", "correctly rounded")], DEFAULT_MARGIN)
    print(f"{fn:8}  verdict: {v.text()}")
