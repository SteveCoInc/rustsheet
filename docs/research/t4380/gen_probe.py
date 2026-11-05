"""T-4380: write probes/t4380-unique-keep-head-holdout, the hold-out of T-4380's change to UNIQUE's merge.

Run from the repo root: python3 -I docs/research/t4380/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4380/predict_engine.py, which records the engine's values as the committed predictions.

T-4380 chose its rule on probes/t4379-pivotby-repeated-unreached-leaf: when the heads of two runs are equal, the
right group joins the left head, which stays the head and meets the next right group too (docs/research/t4241/model.py
`unique_kept`; T-4200's rule took both heads out). The two rules differ only on two-field runs (none of one field up
to length 7). This probe takes runs where they differ, none of them t4379's: every such run of length 4, and a
fixed-seed sample of lengths 5, 6 and 7, plus control runs where they agree on three or more kept keys.
Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B); x < y. Each Calc row is one run, seen through
UNIQUE (B) and GROUPBY with an ARRAYTOTEXT of the positions (C).
"""

import importlib.util
import itertools
import random
import sys

sys.path.insert(0, "docs/research/t4241")
import model  # noqa: E402

_spec = importlib.util.spec_from_file_location("t4241_gen_probe", "docs/research/t4241/gen_probe.py")
assert _spec and _spec.loader
t4241 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4241)

_spec79 = importlib.util.spec_from_file_location("t4379_gen_probe", "docs/research/t4379/gen_probe.py")
assert _spec79 and _spec79.loader
t4379 = importlib.util.module_from_spec(_spec79)
_spec79.loader.exec_module(t4379)

OUT = "eval/suites/probes/cases/t4380-unique-keep-head-holdout.case.json"
KEYS = [k + v for k in "ABC" for v in "xy"]
SAMPLE = {5: 12, 6: 12, 7: 10}
CONTROLS = 8


def runs() -> list[str]:
    rcmp = model.make_rcmp([1, 1])
    seen = {run for run, _ in t4379.RUNS}
    rng = random.Random(4380)
    differ: dict[int, list[str]] = {}
    agree: list[str] = []
    for n in range(4, 8):
        for keys in itertools.product(KEYS, repeat=n):
            run = " ".join(keys)
            rows = [tuple(k) for k in keys]
            new, old = model.unique_kept(rows, rcmp), model.unique_kept(rows, rcmp, t4200=True)
            if run in seen:
                continue
            if new != old:
                differ.setdefault(n, []).append(run)
            elif n >= 5 and len(new) >= 3:
                agree.append(run)
    out = list(differ[4])
    for n, k in SAMPLE.items():
        out += rng.sample(differ[n], k)
    return out + rng.sample(agree, CONTROLS)


def main() -> None:
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    row = 1
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    all_runs = runs()
    for r, run in enumerate(all_runs, start=1):
        keys = run.split()
        a, b = row, row + len(keys) - 1
        for i, key in enumerate(keys):
            data[f"A{row + i}"] = t4241.TEXTS[key[0]]
            data[f"B{row + i}"] = f'="{key[1]}"'
            data[f"D{row + i}"] = i + 1
        row = b + 1
        cols, pos = f"Data!A{a}:B{b}", f"Data!D{a}:D{b}"
        calc[f"A{r}"] = f"run {run}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.UNIQUE({cols}),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({cols},{pos},{agg},0,0),1)"
    t4241.write_case(
        OUT,
        "T-4380: hold-out of T-4380's UNIQUE merge rule (equal run heads: the right group joins the left head, "
        "which stays the head) on two-field runs where it and T-4200's rule differ, none of them t4379's, plus "
        "controls where they agree (A = ab, B = ab&U+00AD, C = U+00AD&ab), through UNIQUE (B) and GROUPBY (C). "
        "Written with the engine's predictions (docs/research/t4380) before the fill.",
        calc,
        data,
    )
    print(len(all_runs), "runs,", len(calc) - len(all_runs), "formulas")


if __name__ == "__main__":
    main()
