"""T-4342: write probes/t4342-groupby-tree-holdout and the T-4241 model's predictions for it, before the fill.

Run from the repo root: python3 -I docs/research/t4342/gen_probe.py. It writes the case and
docs/research/t4342/predictions.tsv (one line per Calc cell: the groups the model predicts, in the canonical
form `compare.py` reads the oracle into). The model is docs/research/t4241/model.py, frozen before this probe was
written (decision 0188 R4: a hold-out for a structural model). Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab
(A = C, B = C, A < B); P = ABC, Q = abc, R = Abc (one class by case). Data per block row: A the first field,
B x/y, C a third field, D the row's position, E 2^(position-1), F p/q (PIVOTBY's column field).

Calc rows:
- 1-30: seeded five-row runs, two fields (A:B). B: GROUPBY ascending, ARRAYTOTEXT of the positions; C: sort_order
  -1; D: SUM of E; E: PIVOTBY by F, positions (no totals).
- 31-50: seeded five-row runs, three fields (A:C, the third a text too). B, C, D and E as above.
- 51-56: ASCII case spellings beside x/y (two fields), GROUPBY ascending only (B): which spelling a group shows
  when a later spelling sorts first on the whole row.
"""

import importlib.util
import json
import random
import sys

sys.path.insert(0, "docs/research/t4241")
import model  # noqa: E402  (the frozen T-4241 model)

# T-4241's generator by path: both are named gen_probe.
_spec = importlib.util.spec_from_file_location("t4241_gen_probe", "docs/research/t4241/gen_probe.py")
assert _spec and _spec.loader
t4241_probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4241_probe)

TEXTS = {
    "A": '="ab"',
    "B": '="ab"&UNICHAR(173)',
    "C": '=UNICHAR(173)&"ab"',
    "P": '="ABC"',
    "Q": '="abc"',
    "R": '="Abc"',
}
OUT = "eval/suites/probes/cases/t4342-groupby-tree-holdout.case.json"
PRED = "docs/research/t4342/predictions.tsv"
SHOW = {"A": "ab", "B": "ab­", "C": "­ab", "P": "ABC", "Q": "abc", "R": "Abc"}

_base_tcmp = model.tcmp


def tcmp(a: str, b: str) -> int:
    """The model's `=`/`<`, with P, Q, R one case-folded class (they never meet A, B or C in a block)."""
    if {a, b} <= {"P", "Q", "R"}:
        return 0
    return _base_tcmp(a, b)


model.tcmp = tcmp
# T-4342's revision (after the fill): the lower-bound search over [0, n-1]. False for the frozen predictions.
INCLUSIVE = False


def runs(rng: random.Random, count: int, fields: int) -> list[list[tuple[str, ...]]]:
    out: list[list[tuple[str, ...]]] = []
    seen = set()
    while len(out) < count:
        texts = [rng.choice("ABC") for _ in range(5)]
        if set(texts) != set("ABC"):
            continue
        xy = [rng.choice("xy") for _ in range(5)]
        if len(set(xy)) < 2:
            continue
        cols = [texts, xy] + ([[rng.choice("ABC") for _ in range(5)]] if fields == 3 else [])
        rows = [tuple(c[i] for c in cols) for i in range(5)]
        if tuple(rows) in seen:
            continue
        seen.add(tuple(rows))
        out.append(rows)
    return out


def shown(key: tuple[str, ...]) -> list[str]:
    return [SHOW.get(k, k) for k in key]


def predict_groupby(rows: list[tuple[str, ...]], dirs: list[int], total: bool) -> str:
    groups = model.groupby(rows, dirs, INCLUSIVE)
    if total:
        return json.dumps(
            [shown(k) + [sum(2 ** (m - 1) for m in ms)] for k, ms in groups], ensure_ascii=False
        )
    return json.dumps([shown(k) + [ms] for k, ms in groups], ensure_ascii=False)


def predict_pivotby(rows: list[tuple[str, ...]], pq: list[str]) -> str:
    groups = model.groupby(rows, [1] * len(rows[0]), INCLUSIVE)
    cols = sorted(set(pq))
    out = []
    for k, ms in groups:
        out.append(shown(k) + [[m for m in ms if pq[m - 1] == c] for c in cols])
    return json.dumps({"columns": cols, "rows": out}, ensure_ascii=False)


def main() -> None:
    rng = random.Random("T-4342")
    blocks: list[tuple[str, list[tuple[str, ...]], bool]] = []
    for rows in runs(rng, 30, 2):
        blocks.append(("2f", rows, True))
    for rows in runs(rng, 20, 3):
        blocks.append(("3f", rows, True))
    for rows in [
        [("P", "y"), ("Q", "x")],
        [("Q", "y"), ("P", "x"), ("R", "x")],
        [("P", "y"), ("Q", "x"), ("R", "y")],
        [("R", "y"), ("P", "y"), ("Q", "x")],
        [("P", "x"), ("Q", "y")],
        [("Q", "y"), ("R", "x"), ("P", "y"), ("Q", "x")],
    ]:
        blocks.append(("case", rows, False))
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    preds: list[tuple[str, str]] = []
    row = 1
    for r, (kind, rows, full) in enumerate(blocks, start=1):
        a, b = row, row + len(rows) - 1
        pq = [rng.choice("pq") for _ in rows]
        if len(set(pq)) < 2:
            pq[0] = "q" if pq[0] == "p" else "p"
        for i, key in enumerate(rows):
            n = row + i
            data[f"A{n}"] = TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            if len(key) == 3:
                data[f"C{n}"] = TEXTS[key[2]]
            data[f"D{n}"] = i + 1
            data[f"E{n}"] = 2**i
            data[f"F{n}"] = f'="{pq[i]}"'
        last = "C" if len(rows[0]) == 3 else "B"
        keys, pos, pw, cf = f"Data!A{a}:{last}{b}", f"Data!D{a}:D{b}", f"Data!E{a}:E{b}", f"Data!F{a}:F{b}"
        calc[f"A{r}"] = kind + " " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
        nf = len(rows[0])
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},LAMBDA(v,ARRAYTOTEXT(v)),0,0),1)"
        preds.append((f"B{r}", predict_groupby(rows, [1] * nf, False)))
        if full:
            calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},LAMBDA(v,ARRAYTOTEXT(v)),0,0,-1),1)"
            preds.append((f"C{r}", predict_groupby(rows, [-1] + [1] * (nf - 1), False)))
            calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},LAMBDA(v,SUM(v)),0,0),1)"
            preds.append((f"D{r}", predict_groupby(rows, [1] * nf, True)))
            calc[f"E{r}"] = (
                f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{cf},{pos},LAMBDA(v,ARRAYTOTEXT(v)),0,0,,0),1)"
            )
            preds.append((f"E{r}", predict_pivotby(rows, pq)))
        row = b + 1
    t4241_probe.write_case(
        OUT,
        "T-4342: a hold-out for T-4241's GROUPBY tree model on intransitive runs (A = ab, "
        "B = ab&U+00AD, C = U+00AD&ab; A = C = B, A < B). Rows 1-30: seeded five-row runs, two fields; rows 31-50: "
        "three fields (text, x/y, text); rows 51-56: ASCII case spellings beside x/y. B: GROUPBY ascending, "
        "ARRAYTOTEXT of the member positions; C: sort_order -1; D: SUM of 2^(position-1); E: PIVOTBY by a p/q "
        "column field, no totals. Written with the model's predictions (docs/research/t4342) before the fill.",
        calc,
        data,
    )
    with open(PRED, "w") as fh:
        fh.write(
            "# T-4342: docs/research/t4241/model.py's predictions for probes/t4342-groupby-tree-holdout, "
            "written before the oracle fill (cell, canonical groups)\n"
        )
        for c, p in preds:
            fh.write(f"{c}\t{p}\n")
    print(len(blocks), "blocks,", len(preds), "predictions")


if __name__ == "__main__":
    main()
