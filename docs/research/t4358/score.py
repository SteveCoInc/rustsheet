"""T-4358: score the fill of probes/t4358-pivotby-unreached-column-leaf.

Run from the repo root: python3 -I docs/research/t4358/score.py. Prints the committed engine predictions'
(predictions.tsv) score by column, and checks a candidate for the column subtotals that the predictions miss: a
first-field subtotal column holds the row group's records whose descent reached that class (docs/research/t4241
model, T-4344's search), whether or not they reached a leaf under it, and not a leaf no record reached's extra
records. The candidate was chosen on these cells, so it owes its own hold-out (the follow-up task).
"""

import importlib.util
import json
import re

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

CASE = "eval/suites/probes/cases/t4358-pivotby-unreached-column-leaf"


def first_classes(rows: list[tuple[str, ...]]) -> tuple[list[str], list[int | None]]:
    """The first field's class keys of the ascending column tree, and each row's class (None: no class)."""
    keys = gen.model.natural_sort(
        [rows[i] for i in gen.model.unique_kept(rows, gen.model.make_rcmp([1, 1]))], gen.model.make_rcmp([1, 1])
    )
    reps: list[str] = []
    for k in keys:
        if not reps or gen.tcmp(reps[-1], k[0]) != 0:
            reps.append(k[0])
    found: list[int | None] = []
    for row in rows:
        i = gen.h3(reps, row[0], 1)
        found.append(i if gen.tcmp(reps[i], row[0]) == 0 else None)
    return reps, found


def main() -> None:
    with open(CASE + ".case.json") as fh:
        calc = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["Calc"]
    with open(CASE + ".oracle.json") as fh:
        oracle = json.load(fh)["sheets"]["Calc"]
    preds = {}
    with open("docs/research/t4358/predictions.tsv") as fh:
        for line in fh:
            if not line.startswith("#"):
                cell, value = line.rstrip("\n").split("\t", 1)
                preds[cell] = json.loads(value)
    for col in "BCDE":
        cells = [c for c in preds if c[0] == col]
        bad = [c for c in cells if preds[c] != oracle[c]["v"]]
        print(f"{col}: {len(cells) - len(bad)} / {len(cells)}", bad)
    ok = total = 0
    for r in range(1, 26):
        label = calc[f"A{r}"]
        keys, pq = label[4:].split(" | ")
        rows = [tuple(k) for k in keys.split()]
        reps, found = first_classes(rows)
        table = [re.findall(r'"([^"]*)"', line) for line in oracle[f"D{r}"]["v"].strip("{}").split(";")]
        header, sub = table[0], table[1]
        subtotal_columns = [j for j in range(1, len(sub)) if sub[j] == "" and header[j] != "Grand Total"]
        for line in table[2:]:
            group = line[0]
            for k, j in enumerate(subtotal_columns):
                want = ", ".join(str(i + 1) for i in range(len(rows)) if pq[i] == group and found[i] == k)
                total += 1
                ok += line[j] == want
    print(f"subtotal candidate: {ok} / {total} subtotal cells")


if __name__ == "__main__":
    main()
