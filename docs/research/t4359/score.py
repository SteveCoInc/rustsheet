"""T-4359: score the hypotheses of model.py (hypotheses.tsv) and the engine's predictions (predictions.tsv)
against probes/t4359-groupby-value-sort-parents' oracle values. Run from the repo root after the fill:
python3 -I docs/research/t4359/score.py.
"""

import importlib.util
import json

_spec = importlib.util.spec_from_file_location("t4359_gen_probe", "docs/research/t4359/gen_probe.py")
assert _spec and _spec.loader
probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(probe)

CASE = "eval/suites/probes/cases/t4359-groupby-value-sort-parents"
HYPS = ("R0", "R1", "R2")
LATER = ("R3",)  # hypotheses added after the fill: scored by recomputing, not from hypotheses.tsv


def parse(text: str) -> list:
    """An ARRAYTOTEXT (strict) text of GROUPBY's groups as [keys..., value] lists."""
    out = []
    for line in text.strip("{}").split(";"):
        cells = json.loads("[" + line + "]")
        out.append(cells)
    return out


def same(a: list, b: list) -> bool:
    if len(a) != len(b):
        return False
    for x, y in zip(a, b):
        if x[:-1] != y[:-1] or abs(float(x[-1]) - float(y[-1])) > 1e-9 * max(1.0, abs(float(y[-1]))):
            return False
    return True


def main() -> None:
    with open(CASE + ".oracle.json") as fh:
        oracle = {c: v["v"] for c, v in json.load(fh)["sheets"]["Calc"].items()}
    hits = dict.fromkeys(HYPS + LATER, 0)
    lines = 0
    with open("docs/research/t4359/hypotheses.tsv") as fh:
        for line in fh:
            if line.startswith("#"):
                continue
            cell, *preds = line.rstrip("\n").split("\t")
            lines += 1
            got = parse(oracle[cell])
            for h, p in zip(HYPS, preds):
                if same(json.loads(p), got):
                    hits[h] += 1
                else:
                    print(f"{h} misses {cell}: {oracle[cell]}")
    with open(CASE + ".case.json") as fh:
        calc = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["Calc"]
    for h in LATER:
        n = 0
        for cell in sorted(c for c in oracle if c[0] in probe.COLUMNS):
            rows = [tuple(k) for k in calc["A" + cell[1:]].split()[1:]]
            if same(probe.predict(rows, cell[0], h), parse(oracle[cell])):
                n += 1
            else:
                print(f"{h} misses {cell}: {oracle[cell]}")
        hits[h] = n
    engine = 0
    with open("docs/research/t4359/predictions.tsv") as fh:
        rows = [line.rstrip("\n").split("\t") for line in fh if not line.startswith("#")]
    for cell, value in rows:
        engine += json.loads(value) == oracle[cell]
    print({h: f"{n}/{lines}" for h, n in hits.items()}, f"engine {engine}/{len(rows)}")


if __name__ == "__main__":
    main()
