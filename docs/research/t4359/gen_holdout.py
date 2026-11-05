"""T-4359: write probes/t4359-groupby-value-sort-holdout, the hold-out for R3 (model.py), chosen after the fill of
probes/t4359-groupby-value-sort-parents.

Run from the repo root: python3 -I docs/research/t4359/gen_holdout.py, then (after `make build` with T-4359's
change, before the fill) python3 -I docs/research/t4359/predict_holdout.py, which commits the engine's values (R3)
as the predictions. Fresh seeded runs (seed "T-4359 hold-out", none of the first probe's), 12 of two fields and 18
of three, picked where R0, R1, R2 and R3 order the groups differently, round-robin over the patterns. Columns B to
E as gen_probe.py's; F: SUM of E with subtotals at every level, sorted by the keys ascending (exploratory: whether
a key sort's subtotal holds the rows that reached its class too; the engine shows the union of its groups').
"""

import importlib.util
import json
import random

_spec = importlib.util.spec_from_file_location("t4359_gen_probe", "docs/research/t4359/gen_probe.py")
assert _spec and _spec.loader
probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(probe)
gen = probe.gen

OUT = "eval/suites/probes/cases/t4359-groupby-value-sort-holdout.case.json"
FIRST = "eval/suites/probes/cases/t4359-groupby-value-sort-parents.case.json"
probe.HYPS = ("R0", "R1", "R2", "R3")


def main() -> None:
    rng = random.Random("T-4359 hold-out")
    with open(FIRST) as fh:
        calc0 = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["Calc"]
    seen = {tuple(tuple(k) for k in v.split()[1:]) for c, v in calc0.items() if c.startswith("A")}
    blocks = probe.pick(rng, 2, 12, seen) + probe.pick(rng, 3, 18, seen)
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    for r, (rows, _) in enumerate(blocks, start=1):
        a, b = 5 * r - 4, 5 * r
        nf = len(rows[0])
        for i, key in enumerate(rows):
            n = a + i
            data[f"A{n}"] = gen.t4342.TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            if nf == 3:
                data[f"C{n}"] = gen.t4342.TEXTS[key[2]]
            data[f"D{n}"] = i + 1
            data[f"E{n}"] = 2**i
        last = "C" if nf == 3 else "B"
        keys, pw = f"Data!A{a}:{last}{b}", f"Data!E{a}:E{b}"
        calc[f"A{r}"] = f"{nf}f " + " ".join("".join(k) for k in rows)
        sm, av = "LAMBDA(v,SUM(v))", "LAMBDA(v,AVERAGE(v))"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,0,{nf + 1}),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,0,{-(nf + 1)}),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{av},0,0,{nf + 1}),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,{nf},{nf + 1}),1)"
        calc[f"F{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,{nf},1),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4359: the hold-out for R3 (docs/research/t4359/model.py), a value sort's parent holding the rows that "
        "reached its class (every row when none did), on intransitive runs (A = ab, B = ab&U+00AD, C = U+00AD&ab): "
        "12 fresh seeded two-field and 18 three-field runs; B to E as t4359-groupby-value-sort-parents, F a key "
        "sort with subtotals. Written with the engine's predictions (docs/research/t4359) before the fill.",
        calc,
        data,
    )
    print(len(blocks), "blocks")


if __name__ == "__main__":
    main()
