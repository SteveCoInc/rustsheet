"""T-4358: write probes/t4358-pivotby-unreached-column-leaf, a hold-out for PIVOTBY's unreached column leaves.

Run from the repo root: python3 -I docs/research/t4358/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4358/predict_engine.py, which records the engine's values as the committed predictions.

The rule under test (chosen on probes/t4344-groupby-descending-search Calc!D39, D41, E38, E41): the column fields
group as GROUPBY's tree does (docs/research/t4241/model.py with T-4344's search), and a column leaf that no record
reaches holds every record of the row group, as GROUPBY's such group holds every row (step 4); a reached leaf holds
the row group's records that reached it.

Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B). Data per block row: A the text, B x/y, D the
row's position, F p/q. Each Calc row is one seeded five-row run whose column tree (text, then x/y) has a leaf no
record reaches in the ascending or the descending column order:
- B: PIVOTBY rows p/q (Data!F), columns text and x/y (Data!A:B), ARRAYTOTEXT of the positions, ascending;
- C: the same with col_sort_order -1;
- D: the same as B with column subtotals (col_total_depth 2): a subtotal column over a leaf no record reaches and
  a reached one (the engine counts a record once);
- E: columns x/y then the text (HSTACK(Data!B, Data!A)), ascending.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

OUT = "eval/suites/probes/cases/t4358-pivotby-unreached-column-leaf.case.json"
BLOCKS = 25


def has_unreached(rows: list[tuple[str, ...]], dirs: list[int]) -> bool:
    """Whether the column tree has a leaf no row reaches: reached leaves hold disjoint rows, and a leaf none reaches
    holds every row, so the members add up to more than the rows exactly then."""
    groups = gen.groupby(rows, dirs, gen.h3)
    return sum(len(ms) for _, ms in groups) > len(rows)


def main() -> None:
    rng = random.Random("T-4358")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    runs = gen.runs(rng, 2)
    row, r = 1, 0
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    while r < BLOCKS:
        rows = next(runs)
        swapped = [(k[1], k[0]) for k in rows]
        if tuple(rows) in seen or not (
            has_unreached(rows, [1, 1]) or has_unreached(rows, [-1, 1]) or has_unreached(swapped, [1, 1])
        ):
            continue
        seen.add(tuple(rows))
        pq = [rng.choice("pq") for _ in rows]
        if len(set(pq)) < 2:
            pq[0] = "q" if pq[0] == "p" else "p"
        r += 1
        a, b = row, row + len(rows) - 1
        for i, key in enumerate(rows):
            n = row + i
            data[f"A{n}"] = gen.t4342.TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            data[f"D{n}"] = i + 1
            data[f"F{n}"] = f'="{pq[i]}"'
        row = b + 1
        calc[f"A{r}"] = "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
        rf, pos = f"Data!F{a}:F{b}", f"Data!D{a}:D{b}"
        cols = f"Data!A{a}:B{b}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{agg},0,0,,0),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{agg},0,0,,0,-1),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{agg},0,0,,2),1)"
        swap = f"_xlfn.HSTACK(Data!B{a}:B{b},Data!A{a}:A{b})"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{swap},{pos},{agg},0,0,,0),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4358: a hold-out for PIVOTBY's column leaves on intransitive runs (A = ab, B = ab&U+00AD, "
        "C = U+00AD&ab; A = C = B, A < B): 25 seeded five-row runs whose column tree has a leaf no record reaches, "
        "by p/q rows, ascending (B), col_sort_order -1 (C), with column subtotals (D) and with x/y as the first "
        "column field (E). Written with the engine's predictions (docs/research/t4358) before the fill.",
        calc,
        data,
    )
    print(r, "blocks,", len(calc) - r, "formulas")


if __name__ == "__main__":
    main()
