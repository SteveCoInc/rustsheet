"""T-4382: score the fill of probes/t4382-pivotby-intransitive-paths.

Run from the repo root: python3 -I docs/research/t4382/score.py. Prints the committed engine predictions'
(predictions.tsv) score by block range and column, then checks one candidate rule on the cells the predictions
miss, over two-field runs (the first-field subtotals):
- a subtotal, on either axis, holds the line's records that reached its class (whether or not they reached a
  leaf under it: T-4378's column rule), or every record of the line when no record reached the class (as a leaf
  no record reaches does, coercion.md step 4);
- relative_to 3 and 4 take the parent subtotal of the cell's own column or row by the same rule, so the two-
  argument LAMBDA's second argument is that subtotal's records (an empty set: `#VALUE!`).
The rule was chosen on these cells, so it owes its own hold-out.
"""

import importlib.util
import json
import re

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

CASE = "eval/suites/probes/cases/t4382-pivotby-intransitive-paths"


def first_classes(rows: list[tuple[str, ...]], first: int) -> tuple[int, list[int | None]]:
    """The number of first-field classes of the tree with the first field in direction `first`, and each row's
    class (None: the search found a class not `=` to it)."""
    order = gen.model.make_rcmp([first, 1])
    keys = gen.model.natural_sort([rows[i] for i in gen.model.unique_kept(rows, gen.model.make_rcmp([1, 1]))], order)
    reps: list[str] = []
    for k in keys:
        if not reps or gen.tcmp(reps[-1], k[0]) != 0:
            reps.append(k[0])
    found: list[int | None] = []
    for row in rows:
        i = gen.h3(reps, row[0], first)
        found.append(i if gen.tcmp(reps[i], row[0]) == 0 else None)
    return len(reps), found


def subtotal(found: list[int | None], line: set[int], k: int) -> str:
    """Class k's subtotal records of a line (1-based record numbers)."""
    hit = [i + 1 for i, c in enumerate(found) if c == k]
    if not hit:
        return ", ".join(map(str, sorted(line)))
    return ", ".join(str(i) for i in hit if i in line)


def table(value: str) -> list[list[str]]:
    """An ARRAYTOTEXT (strict) table's cells, texts unquoted, numbers and errors as written."""
    return [re.findall(r'"([^"]*)"|([^,"]+)', line) and [a or b for a, b in re.findall(r'"([^"]*)"|([^,"]+)', line)]
            for line in value.strip("{}").split(";")]


def main() -> None:
    with open(CASE + ".case.json") as fh:
        sheets = json.load(fh)["sheets"]
    calc = next(s["cells"] for s in sheets if s["name"] == "Calc")
    with open(CASE + ".oracle.json") as fh:
        oracle = json.load(fh)["sheets"]["Calc"]
    preds = {}
    with open("docs/research/t4382/predictions.tsv") as fh:
        for line in fh:
            if not line.startswith("#"):
                cell, value = line.rstrip("\n").split("\t", 1)
                preds[cell] = json.loads(value)
    for lo, hi, cols in ((1, 15, "BCD"), (16, 30, "BCDE"), (31, 42, "BC")):
        for col in cols:
            cells = [f"{col}{r}" for r in range(lo, hi + 1)]
            bad = [c for c in cells if preds[c] != str(oracle[c]["v"])]
            print(f"rows {lo}-{hi} {col}: {len(cells) - len(bad)} / {len(cells)}", bad)
    ok = total = 0
    for r in range(1, 31):
        keys, pq = calc[f"A{r}"][4:].split(" ! ")[0].split(" | ")
        rows = [tuple(k) for k in keys.split()]
        groups = {g: {i + 1 for i in range(len(rows)) if pq[i] == g} for g in "pq"}
        if r <= 15:
            # D: the parent set of each leaf cell under a row group's first-field subtotal.
            n, found = first_classes(rows, 1)
            t = table(str(oracle[f"D{r}"]["v"]))
            subs = [j for j in range(1, len(t[1]) - 1) if t[1][j] == ""]
            assert len(subs) == n
            for line in t[2:]:
                start = 1
                for k, j in enumerate(subs):
                    want = subtotal(found, groups[line[0]], k)
                    for jj in range(start, j):
                        if line[jj] in ("", "#VALUE!"):
                            got_parent = None if line[jj] == "" else ""
                        else:
                            got_parent = line[jj].split(" / ")[1]
                        if got_parent is None:
                            continue
                        total += 1
                        ok += got_parent == want or (got_parent == "" and want == "")
                    start = j + 1
            continue
        # B (ascending) and C (row_sort_order -1): the row subtotals' cells in the p, q and Total columns.
        for col, first in (("B", 1), ("C", -1)):
            n, found = first_classes(rows, first)
            t = table(str(oracle[f"{col}{r}"]["v"]))
            subs = [line for line in t[1:-1] if line[1] == ""]
            assert len(subs) == n
            for k, line in enumerate(subs):
                for j, g in ((2, groups["p"]), (3, groups["q"]), (4, groups["p"] | groups["q"])):
                    total += 1
                    ok += line[j] == subtotal(found, g, k)
    print(f"candidate: {ok} / {total} subtotal and parent cells")


if __name__ == "__main__":
    main()
