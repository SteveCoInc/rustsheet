"""T-4401: write probes/t4401-pivotby-level-subtotal-holdout, a hold-out for PIVOTBY's deeper column subtotals on
intransitive runs.

Run from the repo root: python3 -I docs/research/t4401/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4401/gen_probe.py --predict, which writes the engine's values to predictions.tsv here
with docs/research/t4344/predict_engine.py's `main` (the predictions committed before the fill).

The rule under test (chosen on probes/t4382-pivotby-intransitive-paths Calc!B31:C42): a column subtotal whose class
no record reached holds every record of the line, as a leaf no record reaches does; one whose class a record
reached holds the line's records that reached it (T-4378). Texts A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C,
B = C, A < B). Fresh seeded five-row runs (seed "T-4401") whose column tree has a leaf no record reaches or a record
in no leaf, ascending or with the first field descending, by p/q rows:
- Calc rows 1-16, three column fields (text, x/y, text; col_total_depth 3): B ascending, C col_sort_order -1;
- Calc rows 17-30, four column fields (text, x/y, text, x/y; col_total_depth 4): B ascending, C col_sort_order -1.
Data per block row: A the first text, B x/y, C the second text, D the second x/y (four-field blocks), E the row's
position, F p/q.
"""

import importlib.util
import random
import sys

_spec = importlib.util.spec_from_file_location("t4382_gen_probe", "docs/research/t4382/gen_probe.py")
assert _spec and _spec.loader
t4382 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4382)
gen = t4382.gen

OUT = "eval/suites/probes/cases/t4401-pivotby-level-subtotal-holdout.case.json"
SHOW = t4382.SHOW


def run(rng: random.Random, fields: int) -> list[tuple[str, ...]]:
    """Five rows: every text of A, B and C in the first field, both of x and y in the second."""
    while True:
        cols = []
        for f in range(fields):
            alphabet = "ABC" if f % 2 == 0 else "xy"
            cols.append([rng.choice(alphabet) for _ in range(5)])
        if set(cols[0]) == set("ABC") and len(set(cols[1])) == 2:
            return [tuple(c[i] for c in cols) for i in range(5)]


def main() -> None:
    rng = random.Random("T-4401")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    r = 0
    a = 1
    for fields, last in ((3, 16), (4, 30)):
        while r < last:
            rows = run(rng, fields)
            if tuple(rows) in seen or not t4382.interesting(rows):
                continue
            seen.add(tuple(rows))
            pq = t4382.p_or_q(rng, len(rows))
            r += 1
            for i, key in enumerate(rows):
                n = a + i
                data[f"A{n}"] = gen.t4342.TEXTS[key[0]]
                data[f"B{n}"] = f'="{key[1]}"'
                data[f"C{n}"] = gen.t4342.TEXTS[key[2]]
                if fields == 4:
                    data[f"D{n}"] = f'="{key[3]}"'
                data[f"E{n}"] = i + 1
                data[f"F{n}"] = f'="{pq[i]}"'
            b = a + 4
            end = "C" if fields == 3 else "D"
            rf, cols, pos = f"Data!F{a}:F{b}", f"Data!A{a}:{end}{b}", f"Data!E{a}:E{b}"
            calc[f"A{r}"] = "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
            head = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{SHOW},0,0,,{fields}"
            calc[f"B{r}"] = head + "),1)"
            calc[f"C{r}"] = head + ",-1),1)"
            a = b + 1
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4401: a hold-out for PIVOTBY's deeper column subtotals on intransitive runs (A = ab, B = ab&U+00AD, "
        "C = U+00AD&ab): 16 seeded five-row runs with three column fields (col_total_depth 3) and 14 with four "
        "(col_total_depth 4) by p/q rows, ascending (B) and col_sort_order -1 (C). Written with the engine's "
        "predictions (docs/research/t4401) before the fill.",
        calc,
        data,
    )
    print(r, "blocks,", len(calc) - r, "formulas")


def predict() -> None:
    """The engine's values for the case, before the fill (docs/research/t4344/predict_engine.py)."""
    spec = importlib.util.spec_from_file_location("t4344_predict", "docs/research/t4344/predict_engine.py")
    assert spec and spec.loader
    engine = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(engine)
    engine.CASE = OUT.removesuffix(".case.json")
    engine.OUT = "docs/research/t4401/predictions.tsv"
    engine.main()


if __name__ == "__main__":
    predict() if sys.argv[1:] == ["--predict"] else main()
