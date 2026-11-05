"""R4.1 hold-out of T-4370 (decision 0188): a pooled 30% sha256 split of every cell of the six t4323/t4332 fraction
cases and the two ulp maps, with T-4323's margin of 10 points; exact text rates of model.py's 15-digit digits, of
T-4332's field-by-field cut and of the correctly rounded value (the double rounded exactly to the shown digit).
Run from this directory: `python3 holdout.py`."""
import hashlib
import math
from fractions import Fraction

from model import CASES, chain_fields, fmt, load, model_fields  # model.py puts ../t4332 on the path


def correctly_rounded(x, s):
    units = math.floor(Fraction(x) * 86400 * s + Fraction(1, 2))
    return units % s, (units // s) % 86400, units // s


rows = load([("../t4332/" + m, c) for m, c in CASES] + [("../t4332/meta6.json", "t4332-fraction-fresh"),
                                                       ("meta7.json", "t4370-ulp-map"),
                                                       ("meta8.json", "t4370-ulp-map-2")])


def held(row):
    return int(hashlib.sha256(row[0].encode()).hexdigest(), 16) % 100 < 30


splits = {"train": [r for r in rows if not held(r)], "hold-out": [r for r in rows if held(r)]}
for name, part in splits.items():
    for label, fn in (("15-digit model", model_fields), ("cut (T-4332)", chain_fields),
                      ("correctly rounded", correctly_rounded)):
        ok = sum(fmt(f, s, fn(x, s)) == e for (_, f, s, x, e) in part)
        print(f"{name:9} {label:18} {ok}/{len(part)} = {100 * ok / len(part):.1f}%")
