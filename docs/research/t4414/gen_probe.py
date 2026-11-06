"""T-4414: write probes/t4414-pivotby-mixed-column-subtotal, PIVOTBY's column subtotal of a class that records of
another row line reach but no record of this line does, on intransitive runs.

Run from the repo root: python3 -I docs/research/t4414/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4344/predict_engine.py eval/suites/probes/cases/t4414-pivotby-mixed-column-subtotal
docs/research/t4414/predictions.tsv, which commits the engine's values as the predictions (the current model:
T-4388's `Grouping::class_reached` for a class no record reached, T-4378's prefix filter otherwise).

T-4388's probe (probes/t4388-pivotby-intransitive-subtotals D and E) read its rule from classes no record reached at
all. Here every block is a seeded five-row three-field run (text, x/y, text; A = ab, B = ab&U+00AD, C = U+00AD&ab)
with p/q row lines (Data!F) chosen so that, ascending or with col_sort_order -1, a class above the leaves (a prefix
of one or two fields) is reached by a record of one line and by no record of the other. Data per block row as in
T-4388: A the text, B x/y, C the third text, D the position, F p/q. Calc columns:
- B: PIVOTBY rows = p/q, columns = the three fields, positions as ARRAYTOTEXT, column subtotals (col_total_depth 3);
- C: B with col_sort_order -1.
Column A labels the run and says in which order (asc, desc) the mixed class occurs.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4388_gen_probe", "docs/research/t4388/gen_probe.py")
assert _spec and _spec.loader
t4388 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4388)
gen = t4388.gen

OUT = "eval/suites/probes/cases/t4414-pivotby-mixed-column-subtotal.case.json"
BLOCKS = 30
ORDERS = {"asc": [1, 1, 1], "desc": [-1, 1, 1]}


def mixed_class(rows: list[tuple[str, ...]], pq: list[str], dirs: list[int]) -> bool:
    """Whether a class above the leaves is reached by a record of one row line and by no record of the other."""
    _, paths = gen.descents(rows, dirs, gen.h3)
    nf = len(rows[0])
    by_line: dict[str, set] = {"p": set(), "q": set()}
    for path, line in zip(paths, pq, strict=True):
        by_line[line] |= {path[:level] for level in range(1, min(len(path), nf - 1) + 1)}
    return bool(by_line["p"] ^ by_line["q"])


def main() -> None:
    rng = random.Random("T-4414")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    runs = gen.runs(rng, 3)
    r = 0
    for _ in range(20000):
        if r == BLOCKS:
            break
        rows = next(runs)
        if tuple(rows) in seen:
            continue
        seen.add(tuple(rows))
        pq = [rng.choice("pq") for _ in rows]
        if len(set(pq)) < 2:
            continue
        hits = [name for name, dirs in ORDERS.items() if mixed_class(rows, pq, dirs)]
        if not hits:
            continue
        a = 5 * r + 1
        r += 1
        b = a + 4
        for i, key in enumerate(rows):
            n = a + i
            data |= {f"A{n}": gen.t4342.TEXTS[key[0]], f"B{n}": f'="{key[1]}"', f"C{n}": gen.t4342.TEXTS[key[2]]}
            data |= {f"D{n}": i + 1, f"F{n}": f'="{pq[i]}"'}
        keys, pq_range, pos = f"Data!A{a}:C{b}", f"Data!F{a}:F{b}", f"Data!D{a}:D{b}"
        calc[f"A{r}"] = "3f " + " ".join("".join(k) for k in rows) + " | " + "".join(pq) + " | " + ",".join(hits)
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({pq_range},{keys},{pos},{agg},0,0,,3),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({pq_range},{keys},{pos},{agg},0,0,,3,-1),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4414: PIVOTBY's column subtotal of a class reached by records of one row line and by none of the other, on "
        "intransitive runs (A = ab, B = ab&U+00AD, C = U+00AD&ab): 30 seeded five-row three-field runs by p/q rows, "
        "column subtotals ascending (B) and col_sort_order -1 (C). Written with the engine's predictions "
        "(docs/research/t4414) before the fill.",
        calc,
        data,
    )
    print(r, "blocks")


if __name__ == "__main__":
    main()
