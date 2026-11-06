"""T-4401: write probes/t4401-pivotby-unreached-subtotal-holdout, the hold-out of the rule that a PIVOTBY column
subtotal of a class no record reached holds every record of its row line (T-4388's finding, T-4382's Calc!B31:C41).

Run from the repo root: python3 -I docs/research/t4401/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4344/predict_engine.py eval/suites/probes/cases/t4401-pivotby-unreached-subtotal-holdout
docs/research/t4401/predictions.tsv, which commits the engine's values as the predictions (written on PR #3430's
head e4c7b2248, T-4388's rule).

Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B). Data per block row: A the text, B x/y, C and D
more texts (D only in four-field blocks), E the row's position, F p/q. Fresh seeded five-row runs (seed "T-4401"),
by p/q rows, with three column fields (A:C, col_total_depth 3: subtotals at levels 1 and 2) in Calc rows 1-12 and
four (A:D, col_total_depth 4: levels 1 to 3) in rows 13-24. A run is kept when, ascending or with the first field
descending, the column tree has a class above the leaves at level 2 or deeper that no record's descent reaches
(docs/research/t4344's `descents`, T-4344's search). B: ARRAYTOTEXT of the positions, ascending; C: the same with
col_sort_order -1.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

OUT = "eval/suites/probes/cases/t4401-pivotby-unreached-subtotal-holdout.case.json"
SHOW = "LAMBDA(v,ARRAYTOTEXT(v))"
BLOCKS = 12


def runs(rng: random.Random, fields: int):
    """Endless seeded five-row runs of `fields` fields: a text (each of A, B, C present), x/y (both present), and
    texts for the rest."""
    while True:
        first = [rng.choice("ABC") for _ in range(5)]
        xy = [rng.choice("xy") for _ in range(5)]
        if set(first) != set("ABC") or len(set(xy)) < 2:
            continue
        cols = [first, xy] + [[rng.choice("ABC") for _ in range(5)] for _ in range(fields - 2)]
        yield [tuple(c[i] for c in cols) for i in range(5)]


def unreached_inner_class(rows: list[tuple[str, ...]], first: int) -> bool:
    """Whether the column tree (first field in direction `first`, the rest ascending) has a class at level 2 or
    deeper, above the leaves, that no record's descent reaches."""
    nf = len(rows[0])
    leaves, paths = gen.descents(rows, [first] + [1] * (nf - 1), gen.h3)
    inner = {path[:k] for path, _ in leaves for k in range(2, nf)}
    reached = {path[:k] for path in paths for k in range(2, len(path) + 1)}
    return bool(inner - reached)


def main() -> None:
    rng = random.Random("T-4401")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    row = 1
    r = 0
    for fields, last in ((3, "C"), (4, "D")):
        source = runs(rng, fields)
        kept = 0
        while kept < BLOCKS:
            rows = next(source)
            if tuple(rows) in seen or not (unreached_inner_class(rows, 1) or unreached_inner_class(rows, -1)):
                continue
            seen.add(tuple(rows))
            kept += 1
            pq = [rng.choice("pq") for _ in rows]
            if len(set(pq)) < 2:
                pq[0] = "q" if pq[0] == "p" else "p"
            a = row
            for i, key in enumerate(rows):
                n = a + i
                data[f"A{n}"] = gen.t4342.TEXTS[key[0]]
                data[f"B{n}"] = f'="{key[1]}"'
                for col, k in zip("CD", key[2:]):
                    data[f"{col}{n}"] = gen.t4342.TEXTS[k]
                data[f"E{n}"] = i + 1
                data[f"F{n}"] = f'="{pq[i]}"'
            row = a + len(rows)
            b = row - 1
            r += 1
            rf, cols, pos = f"Data!F{a}:F{b}", f"Data!A{a}:{last}{b}", f"Data!E{a}:E{b}"
            calc[f"A{r}"] = "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
            calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{SHOW},0,0,,{fields}),1)"
            calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{SHOW},0,0,,{fields},-1),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4401: hold-out of PIVOTBY's column subtotal of a class no record reached on intransitive runs (A = ab, "
        "B = ab&U+00AD, C = U+00AD&ab): fresh seeded runs with three column fields (rows 1-12) and four (13-24), "
        "subtotals at every level, ascending (B) and with col_sort_order -1 (C). Written with the engine's "
        "predictions (docs/research/t4401) before the fill.",
        calc,
        data,
    )
    print(r, "blocks,", len(calc) - r, "formulas")


if __name__ == "__main__":
    main()
