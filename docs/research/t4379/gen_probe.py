"""T-4379: write probes/t4379-pivotby-repeated-unreached-leaf, which asks why Excel leaves out a repeated column leaf
no record reaches over Cx Ay Ax Bx Ax (probes/t4358-pivotby-unreached-column-leaf Calc!B6) but shows it over
Cx Cx Bx Ay Bx (probes/t4344-groupby-descending-search Calc!D39) and Cx Ax Cx Ay Bx (t4358 Calc!B11).

Run from the repo root: python3 -I docs/research/t4379/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4379/predict_engine.py, which records the engine's values as the committed predictions.

Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B). Data per block row: A the text, B x/y, D the
row's position, F p/q. Each Calc row is one run (column A labels it), seen through:
- B: UNIQUE of the two fields (which keys UNIQUE keeps: T-4200's merge);
- C: GROUPBY of the two fields with an ARRAYTOTEXT of the positions (whether GROUPBY's tree repeats (C,x) too);
- D: PIVOTBY rows p/q (Data!F), columns the two fields, as t4358's B column;
- E: the same with one row group (every row p): whether the row field matters;
- F: the same as D with col_sort_order -1.
The runs are B6's, its sub-runs and reorders with and without the rows UNIQUE drops (Ax after Cx), and the runs
that show the repeated leaf (D39, B11) for contrast.
"""

import importlib.util

_spec = importlib.util.spec_from_file_location("t4241_gen_probe", "docs/research/t4241/gen_probe.py")
assert _spec and _spec.loader
t4241 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4241)

OUT = "eval/suites/probes/cases/t4379-pivotby-repeated-unreached-leaf.case.json"

# One run per line: the keys, p/q per row, and why it is here. B6 first, then variants.
_RUNS_TEXT = """
Cx Ay Ax Bx Ax | qqppq | t4358 B6
Cx Ay Ax Bx | qqpp | B6 without its last Ax
Cx Ay Bx Ax | qqpq | B6 without its first Ax
Cx Ay Bx | qqp | B6 without the Ax rows UNIQUE drops
Cx Ay Bx Ax Ax | qqppq | B6's Ax rows after Bx
Ax Cx Ay Bx Ax | pqqpq | an Ax first
Cx Ay Cx Bx Cx | qqppq | B6 with its Ax rows as Cx
Cx Ay Ax Bx Cx | qqppq | B6 with its last Ax as Cx
Cx Ay Ax Bx Bx | qqppq | B6 with its last Ax as Bx
Cx Ay Ax Ax Bx | qqppq | B6 with Bx last
Cx Ax Ay Bx Ax | qpqpq | B6 with rows 2 and 3 swapped
Cx Cx Bx Ay Bx | qpqqq | t4344 D39: the repeated leaf shown
Cx Ax Cx Ay Bx | qpqqq | t4358 B11: shown, with an Ax row
Cx Ay Bx Bx | qqpq | D39-like without the second Cx
Cx Cx Ay Bx | qpqq | D39 reordered
"""
RUNS = [tuple(part.strip() for part in line.split("|")[:2]) for line in _RUNS_TEXT.strip().splitlines()]


def main() -> None:
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    row = 1
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    for r, (run, pq) in enumerate(RUNS, start=1):
        keys = run.split()
        a, b = row, row + len(keys) - 1
        for i, key in enumerate(keys):
            n = row + i
            data[f"A{n}"] = t4241.TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            data[f"D{n}"] = i + 1
            data[f"F{n}"] = f'="{pq[i]}"'
            data[f"G{n}"] = '="p"'
        row = b + 1
        cols, pos = f"Data!A{a}:B{b}", f"Data!D{a}:D{b}"
        calc[f"A{r}"] = f"run {run} | {pq}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.UNIQUE({cols}),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({cols},{pos},{agg},0,0),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!F{a}:F{b},{cols},{pos},{agg},0,0,,0),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!G{a}:G{b},{cols},{pos},{agg},0,0,,0),1)"
        calc[f"F{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!F{a}:F{b},{cols},{pos},{agg},0,0,,0,-1),1)"
    t4241.write_case(
        OUT,
        "T-4379: why PIVOTBY leaves out a repeated column leaf no record reaches over Cx Ay Ax Bx Ax (t4358 Calc!B6) "
        "but shows it over Cx Cx Bx Ay Bx (t4344 D39) (A = ab, B = ab&U+00AD, C = U+00AD&ab): B6's run, its "
        "sub-runs and reorders, through UNIQUE (B), GROUPBY (C), PIVOTBY by p/q (D), by one row group (E) and with "
        "col_sort_order -1 (F). Written with the engine's predictions (docs/research/t4379) before the fill.",
        calc,
        data,
    )
    print(len(RUNS), "runs,", len(calc) - len(RUNS), "formulas")


if __name__ == "__main__":
    main()
