"""T-4370's model of the fraction digits of a second (frozen 2026-11-05, before probes/t4370-fraction-fresh).

The clock and elapsed fields stay as T-4332 cut them (docs/research/t4332/chain.py). The fraction digits come from a
separate path: the remainder `r` in [0, 1) of a second left by the same half-step cut (`frac(x + 0.5/(86400*s))` cut
by 24, 60, 60 in plain doubles) is rounded to 15 significant decimal digits, and the digits are `floor(r15*s) mod s`.
When `r15` reaches 1 the digits wrap to zero while the seconds field, cut on its own, does not carry: Excel prints
"41.0" for `TEXT(41.95/86400-2*2^-64,"[s].0")` (probes/t4370-ulp-map-2 A417).

Run from this directory: `python3 model.py` scores it and T-4332's chain on the t4323/t4332 fraction cases and the
two ulp maps; `python3 model.py meta9.json:t4370-fraction-fresh` scores the fresh sweep.
"""
import json
import math
import os
import sys
from decimal import ROUND_FLOOR, Decimal

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "../t4332"))
sys.path.insert(0, os.path.join(HERE, "../t4323"))
from chain import CASES, chain_units, rnd  # noqa: E402
from frozen_fmt import fmt as _fmt  # noqa: E402

EXTRA = {"s.0": "{clock_s}.{fr}", "[h]:mm:ss.0": "{eh}:{em:02d}:{es:02d}.{fr}", "[m]:ss.0": "{el_m}:{es:02d}.{fr}"}


def fmt(f, s, fields):
    """frozen_fmt's text, plus the formats only the ulp maps use."""
    if f not in EXTRA:
        return _fmt(f, s, fields)
    frac, clock, el = fields
    fr = f"{frac:0{len(str(s)) - 1}d}"
    return EXTRA[f].format(clock_s=clock % 60, fr=fr, eh=el // 3600, em=el // 60 % 60, es=el % 60, el_m=el // 60)


def remainder(x, s):
    """The half-step cut's remainder in [0, 1) of a second, in plain doubles."""
    v = x + 0.5 / (86400 * s)
    v -= math.floor(v)
    for f in (24, 60, 60):
        v *= f
        v -= math.floor(v)
    return v


def digits(x, s):
    """floor(r15 * s) mod s, r15 being the remainder rounded to 15 significant digits."""
    r = remainder(x, s)
    r15 = Decimal(f"{r:.14e}") if r else Decimal(0)
    return int((r15 * s).to_integral_value(rounding=ROUND_FLOOR)) % s


def model_fields(x, s):
    u = chain_units(x, s)
    return digits(x, s), u // s, rnd(x * 86400 * s) // s


def chain_fields(x, s):
    u = chain_units(x, s)
    return u % s, u // s, rnd(x * 86400 * s) // s


def load(pairs):
    """(cell id, format, scale, serial, Excel's text). Meta values: [f, s, day, seconds] or [f, s, k, n, serial hex]."""
    rows = []
    for meta_path, case in pairs:
        meta = json.load(open(os.path.join(HERE, meta_path)))
        oracle = json.load(open(os.path.join(HERE, f"../../../eval/suites/probes/cases/{case}.oracle.json")))
        cells = oracle["sheets"]["Sheet1"]
        for c, m in meta.items():
            if len(m) == 5:
                x = float.fromhex(m[4])
            else:
                d, k = m[2], m[3]
                x = (d + k / 86400) if d else k / 86400
            rows.append((f"{case}!{c}", m[0], m[1], x, cells[c]["v"]))
    return rows


def score(rows, fields_fn):
    ok = [r[0] for r in rows if fmt(r[1], r[2], fields_fn(r[3], r[2])) == r[4]]
    return len(ok), len(rows), sorted({r[0] for r in rows} - set(ok))


if __name__ == "__main__":
    groups = [[tuple(a.split(":")) for a in sys.argv[1:]]] if sys.argv[1:] else [
        [("../t4332/" + m, c) for m, c in CASES] + [("../t4332/meta6.json", "t4332-fraction-fresh")],
        [("meta7.json", "t4370-ulp-map"), ("meta8.json", "t4370-ulp-map-2")]]
    for pairs in groups:
        rows = load(pairs)
        for name, fn in (("model", model_fields), ("chain (T-4332)", chain_fields)):
            ok, n, bad = score(rows, fn)
            print(f"{','.join(c for _, c in pairs)[:60]:60} {name:15} {ok}/{n}")
