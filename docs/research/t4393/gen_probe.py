"""T-4393: write probes/t4393-unique-merge-holdout, the hold-out of T-4393's UNIQUE merge model, and its predictions.

Run from the repo root: python3 -I docs/research/t4393/gen_probe.py. It writes the case and
docs/research/t4393/predictions.tsv (the model's UNIQUE keys, column B, and GROUPBY groups, column C, per run),
committed before the oracle fill.

T-4393 chose its model (docs/research/t4393/model.py) on the 60 two-field runs of t4379 and t4380. This probe takes
fresh runs (seed 4393, none of t4379's or t4380's) of lengths 4 to 7 where the model and T-4380's best coalescing
rule `pushkeep` keep different rows, runs where it and T-4200's rule (the engine's) differ but `pushkeep` agrees,
and controls where all three agree on three or more kept keys. Texts and layout as t4380: A = ab,
B = ab&U+00AD, C = U+00AD&ab; each Calc row is one run, through UNIQUE (B) and GROUPBY of the positions (C).
"""

import importlib.util
import itertools
import random

_spec93 = importlib.util.spec_from_file_location("t4393_model", "docs/research/t4393/model.py")
assert _spec93 and _spec93.loader
t4393 = importlib.util.module_from_spec(_spec93)
_spec93.loader.exec_module(t4393)

_spec = importlib.util.spec_from_file_location("t4380_gen_probe", "docs/research/t4380/gen_probe.py")
assert _spec and _spec.loader
t4380 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4380)

OUT = "eval/suites/probes/cases/t4393-unique-merge-holdout.case.json"
PREDICTIONS = "docs/research/t4393/predictions.tsv"
SAMPLE = {"pushkeep": {5: 2, 6: 14, 7: 20}, "old": {5: 6, 6: 6, 7: 6}}
CONTROLS = 6


def runs() -> list[str]:
    rcmp = t4380.model.make_rcmp([1, 1])
    seen = {run for _, _, run, _ in t4393.candidates.cases()}
    rng = random.Random(4393)
    pools: dict[str, dict[int, list[str]]] = {"pushkeep": {}, "old": {}}
    agree: list[str] = []
    for n in range(4, 8):
        for keys in itertools.product(t4380.KEYS, repeat=n):
            run = " ".join(keys)
            if run in seen:
                continue
            rows = [tuple(k) for k in keys]
            new = t4393.unique_kept(rows, rcmp)
            if new != t4380.candidates.kept(rows, rcmp, "pushkeep"):
                pools["pushkeep"].setdefault(n, []).append(run)
            elif new != t4380.candidates.kept(rows, rcmp, "old"):
                pools["old"].setdefault(n, []).append(run)
            elif n >= 5 and len(new) >= 3:
                agree.append(run)
    out: list[str] = []
    for rule, sample in SAMPLE.items():
        for n, k in sample.items():
            out += rng.sample(pools[rule][n], k)
    return out + rng.sample(agree, CONTROLS)


def main() -> None:
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    lines = [
        "# T-4393: the model's predictions for probes/t4393-unique-merge-holdout, written before the oracle fill"
        " (cell, run, UNIQUE keys or GROUPBY groups as key:positions)"
    ]
    row = 1
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    all_runs = runs()
    for r, run in enumerate(all_runs, start=1):
        keys = run.split()
        a, b = row, row + len(keys) - 1
        for i, key in enumerate(keys):
            data[f"A{row + i}"] = t4380.t4241.TEXTS[key[0]]
            data[f"B{row + i}"] = f'="{key[1]}"'
            data[f"D{row + i}"] = i + 1
        row = b + 1
        cols, pos = f"Data!A{a}:B{b}", f"Data!D{a}:D{b}"
        calc[f"A{r}"] = f"run {run}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.UNIQUE({cols}),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({cols},{pos},{agg},0,0),1)"
        groups = t4393.groupby([tuple(k) for k in keys], [1, 1])
        lines.append(f"B{r}\t{run}\t{t4393.unique_keys(run)}")
        lines.append(f"C{r}\t{run}\t" + " ".join("".join(k) + ":" + ",".join(map(str, p)) for k, p in groups))
    t4380.t4241.write_case(
        OUT,
        "T-4393: hold-out of T-4393's UNIQUE merge model (equal run heads: the right group joins the left head; no "
        "coalescing with the last group out) on fresh two-field runs (A = ab, B = ab&U+00AD, C = U+00AD&ab) where "
        "it differs from T-4380's pushkeep or from T-4200's rule, plus controls, through UNIQUE (B) and GROUPBY (C). "
        "Written with the model's predictions (docs/research/t4393/predictions.tsv) before the fill.",
        calc,
        data,
    )
    with open(PREDICTIONS, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(len(all_runs), "runs,", len(calc) - len(all_runs), "formulas")


if __name__ == "__main__":
    main()
