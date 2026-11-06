"""T-4393: a UNIQUE merge model that fits every two-field run of probes/t4379 and probes/t4380 (column B).

Run from the repo root: python3 -I docs/research/t4393/model.py. A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C,
B = C, A < B); x < y; rows compare field by field (docs/research/t4241/model.py `make_rcmp`).

The model (`unique_kept`), a natural merge sort over groups of rows:
1. Runs: a row that `=` the last group of the current run joins that group; a row below it starts a new run.
2. Bottom-up merge of neighbouring runs (as `model.natural_sort`). When the two heads are `=`, the right head's
   group joins the left head's group, which stays the head of its run and meets the next right head too.
3. A group taken out of the merge is NOT compared with the last group already out (T-4200's rule and T-4380's
   `pushkeep` coalesce there): two `=` groups that never meet as heads stay apart.
Kept rows are each group's first row, in input order.

Over Cy Cx Ay Bx Ay (t4380 row 6) Excel's UNIQUE is Cy, Cx, Bx, Ay: runs [Cy] [Cx Ay Bx] [Ay]; the first merge
joins Ay (row 3) to Cy and gives Cx Bx Cy (Bx follows Cx with no coalescing, though Bx = Cx); the second merge
takes Cx, then Ay (row 5) < Bx, so it never meets Cy and stays its own group.

Scores (2026-11-06; `main` prints them): 60 of 60 UNIQUE runs of t4379 and t4380 (candidates.py's rules: T-4200's `old` 33 misses,
`keep` 12, `pushkeep` 5); GROUPBY t4380 column C through docs/research/t4241/model.py's tree 38 of 45 (T-4200's
UNIQUE 23); docs/research/t4241/model.py's 234 blocks of t4195 and t4241 unchanged (234 of 234 under both).
Hold-out (probes/t4393-unique-merge-holdout, predictions committed before the fill): UNIQUE 60 of 60, GROUPBY 32 of 60
(T-4472).
"""

import importlib.util
import json
import sys

sys.path.insert(0, "docs/research/t4241")
import model

_spec = importlib.util.spec_from_file_location("t4380_candidates", "docs/research/t4380/candidates.py")
assert _spec and _spec.loader
candidates = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(candidates)


def unique_kept(rows, rcmp):
    """T-4393's merge: kept row indices in input order (the module docstring's rule)."""

    def merge(a, b):
        a, b, o = list(a), list(b), []
        while a and b:
            c = rcmp(rows[b[0][0]], rows[a[0][0]])
            if c == 0:
                a[0] = a[0] + b.pop(0)
            elif c < 0:
                o.append(b.pop(0))
            else:
                o.append(a.pop(0))
        return o + a + b

    return sorted(min(g) for g in model.bottom_up(model.group_runs(rows, rcmp), merge))


def unique_keys(run):
    """UNIQUE of a run written as keys ("Cy Cx Ay"), in the same form."""
    rows = [tuple(k) for k in run.split()]
    return " ".join("".join(rows[j]) for j in unique_kept(rows, model.make_rcmp([1, 1])))


def groupby(rows, dirs):
    """docs/research/t4241/model.py's GROUPBY tree built on this module's UNIQUE."""
    old = model.unique_kept
    model.unique_kept = unique_kept
    try:
        return model.groupby(rows, dirs)
    finally:
        model.unique_kept = old


def holdout() -> None:
    """Score docs/research/t4393/predictions.tsv against probes/t4393-unique-merge-holdout's fill."""
    names = {"\u00adab": "C", "ab\u00ad": "B", "ab": "A"}
    with open("eval/suites/probes/cases/t4393-unique-merge-holdout.oracle.json", encoding="utf-8") as f:
        oracle = json.load(f)["sheets"]["Calc"]
    with open("docs/research/t4393/predictions.tsv", encoding="utf-8") as f:
        lines = [line for line in f if not line.startswith("#")]
    hits = {"B": [0, 0], "C": [0, 0]}
    for line in lines:
        cell, _run, want = line.rstrip("\n").split("\t")
        raw = oracle[cell]["v"]
        if cell[0] == "B":
            parts = [p.split(",") for p in raw.strip("{}").split(";")]
            got = " ".join(names[t.strip('"')] + f.strip('"') for t, f in parts)
        else:
            got = " ".join("".join(k) + ":" + ",".join(map(str, p)) for k, p in model.parse(raw, 2))
        hits[cell[0]][0] += got == want
        hits[cell[0]][1] += 1
    print("hold-out UNIQUE (B):", *hits["B"], "GROUPBY (C):", *hits["C"])


def main() -> None:
    cases = candidates.cases()
    miss = [(nm, i, run, ex, unique_keys(run)) for nm, i, run, ex in cases if unique_keys(run) != ex]
    print("UNIQUE t4379 + t4380:", len(cases) - len(miss), "/", len(cases))
    for m in miss:
        print("   ", m)
    model.score(groupby, [*model.BLOCKS, ("t4380-unique-keep-head-holdout", "C", 2)])
    holdout()


if __name__ == "__main__":
    main()
