"""T-4349: rank short-range selectors between the cut's forms and the geometric sum (decision 0188 R4.1).

Reads dump.py's table (today's engine as `base`, the engine with `GEOMETRIC_MIN_LEAD` 0 as `geo`). A selector is one
to three conditions, all of which must hold, from these atoms: the type, a rate cut, a lead cut and a period-count
cut. Where it holds and the two builds differ, a cell takes `geo`'s value, else `base`'s. Selectors that put an old
cell (fit `cumulative`) newly visibly off are dropped. The rest are ranked by the bit-exact gain over the cut on the
train side of the old cells, then by the old visibly-off cells fixed, then by fewer conditions; the first is frozen.
The test side and the two T-3524 sweeps are printed beside it but take no part in the ranking.

Run: python3 docs/research/t4349/select.py rows.tsv
"""

from __future__ import annotations

import csv
import itertools
import sys
from collections.abc import Callable

Row = dict[str, str]
Atom = Callable[[Row], bool]
SIDES: dict[str, Callable[[Row], bool]] = {
    "train": lambda r: r["fit"] == "cumulative" and r["holdout"] == "0",
    "test": lambda r: r["fit"] == "cumulative" and r["holdout"] == "1",
    "fresh": lambda r: r["fit"] == "cumulative-fresh",
    "tiny": lambda r: r["fit"] == "cumulative-tiny",
}


def atoms() -> dict[str, Atom]:
    out: dict[str, Atom] = {}
    for t in (0, 1):
        out[f"type={t}"] = lambda r, t=t: float(r["type"]) == t
    for cut in (1e-8, 1e-6, 1e-5, 1e-4, 1e-3, 3e-3, 1e-2, 0.05):
        out[f"rate<{cut:g}"] = lambda r, c=cut: float(r["rate"]) < c
        out[f"rate>={cut:g}"] = lambda r, c=cut: float(r["rate"]) >= c
    for cut in (1e-5, 1e-4, 1e-3, 1e-2, 0.03, 0.095):
        out[f"lead<{cut:g}"] = lambda r, c=cut: float(r["lead"]) < c
        out[f"lead>={cut:g}"] = lambda r, c=cut: float(r["lead"]) >= c
    for n in (1, 2, 4, 8, 16):
        out[f"periods<={n}"] = lambda r, n=n: int(r["count"]) <= n
        out[f"periods>{n}"] = lambda r, n=n: int(r["count"]) > n
    return out


def main() -> None:
    rows = [r for r in csv.DictReader(open(sys.argv[1]), delimiter="\t") if r["base"] != r["geo"]]
    table = atoms()
    ranked = []
    for k in (1, 2, 3):
        for combo in itertools.combinations(table, k):
            gain = dict.fromkeys(SIDES, 0)
            new_visible = fixed_visible = 0
            for r in rows:
                if not all(table[a](r) for a in combo):
                    continue
                delta = (r["geo_ulps"] == "0") - (r["base_ulps"] == "0")
                for side, member in SIDES.items():
                    if member(r):
                        gain[side] += delta
                if r["fit"] == "cumulative":
                    new_visible += r["base_vis"] == "0" and r["geo_vis"] == "1"
                    fixed_visible += r["base_vis"] == "1" and r["geo_vis"] == "0"
            if new_visible == 0 and gain["train"] > 0:
                ranked.append((-gain["train"], -fixed_visible, k, " & ".join(combo), gain, fixed_visible))
    ranked.sort(key=lambda x: x[:3])
    print("selector\ttrain\ttest\tfresh\ttiny\told visibly fixed")
    for *_, name, gain, fixed in ranked[:20]:
        print(f"{name}\t{gain['train']}\t{gain['test']}\t{gain['fresh']}\t{gain['tiny']}\t{fixed}")


if __name__ == "__main__":
    main()
