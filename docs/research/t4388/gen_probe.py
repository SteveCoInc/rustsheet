"""T-4388: write probes/t4388-pivotby-intransitive-subtotals, PIVOTBY's and GROUPBY's subtotals on intransitive runs.

Run from the repo root: python3 -I docs/research/t4388/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4344/predict_engine.py eval/suites/probes/cases/t4388-pivotby-intransitive-subtotals
docs/research/t4388/predictions.tsv, which commits the engine's values as the predictions (the current model:
T-4359's parents for GROUPBY and PIVOTBY's row axis, T-4378's column subtotals, T-4358's column leaves).

30 fresh seeded five-row runs (seed "T-4388"; A = ab, B = ab&U+00AD, C = U+00AD&ab, so A = C = B, A < B), p/q rows
(Data!F): 15 three-field runs (text, x/y, text) whose tree, ascending or with the first or second field descending,
has a class above the leaves no row reaches, and 15 two-field runs (text, x/y) with a leaf no row reaches or a row
in no leaf. Data per block row: A the text, B x/y, C the third text, D the position, E 2^(position-1). Calc columns
(nf the fields):
- B: PIVOTBY rows = the text and x/y, columns = p/q, positions as ARRAYTOTEXT, row subtotals (row_total_depth nf);
- C: the same as SUM of E sorted by the first value column (row_sort_order nf + 1) with row subtotals;
- D: PIVOTBY rows = p/q, columns = the text and x/y, positions as ARRAYTOTEXT, column subtotals (col_total_depth nf):
  a column subtotal of a class no record of the row group reached (T-4378 gives it none);
- E: D with col_sort_order -1;
- F: GROUPBY of the text and x/y, positions as ARRAYTOTEXT, subtotals at every level, sort_order -1;
- G: the same with sort_order -2.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4358_gen_probe", "docs/research/t4358/gen_probe.py")
assert _spec and _spec.loader
t4358 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4358)
gen = t4358.gen

OUT = "eval/suites/probes/cases/t4388-pivotby-intransitive-subtotals.case.json"
BLOCKS = 30


def unreached_class(rows: list[tuple[str, ...]], dirs: list[int]) -> bool:
    """Whether a class above the leaves (a key prefix shorter than the fields) is reached by no row."""
    leaves, paths = gen.descents(rows, dirs, gen.h3)
    nf = len(rows[0])
    classes = {p[:level] for p, _ in leaves for level in range(1, nf)}
    reached = {p[:level] for p in paths for level in range(1, min(len(p), nf - 1) + 1)}
    return bool(classes - reached)


def directions(nf: int) -> list[list[int]]:
    return [[1] * nf, [-1] + [1] * (nf - 1), [1, -1] + [1] * (nf - 2)]


def interesting(rows: list[tuple[str, ...]]) -> bool:
    """A leaf no row reaches or a row in no leaf, ascending or with the first or second field descending."""
    for dirs in directions(len(rows[0])):
        held = {m for _, ms in gen.groupby(rows, dirs, gen.h3) for m in ms}
        if sum(len(ms) for _, ms in gen.groupby(rows, dirs, gen.h3)) > len(rows) or len(held) < len(rows):
            return True
    return False


def pick(rng: random.Random, nf: int, count: int, seen: set, need_class: bool) -> list:
    """`count` distinct seeded runs of `nf` fields that are interesting (and, with `need_class`, have a class above
    the leaves no row reaches), from at most 20000 draws."""
    out: list = []
    runs = gen.runs(rng, nf)
    for _ in range(20000):
        if len(out) == count:
            break
        rows = next(runs)
        if tuple(rows) in seen:
            continue
        seen.add(tuple(rows))
        if need_class and not any(unreached_class(rows, d) for d in directions(nf)):
            continue
        if need_class or interesting(rows):
            out.append(rows)
    return out


def main() -> None:
    rng = random.Random("T-4388")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    sm = "LAMBDA(v,SUM(v))"
    r = 0
    # Half the blocks are three-field runs with a class above the leaves that no row reaches (the column-subtotal
    # question; in two-field runs such a first-field class is rare), half two-field runs of any interesting shape.
    picked = [(rows, 3) for rows in pick(rng, 3, BLOCKS // 2, seen, True)]
    picked += [(rows, 2) for rows in pick(rng, 2, BLOCKS - len(picked), seen, False)]
    for rows, nf in picked:
        pq = [rng.choice("pq") for _ in rows]
        if len(set(pq)) < 2:
            pq[0] = "q" if pq[0] == "p" else "p"
        a = 5 * r + 1
        r += 1
        b = a + 4
        for i, key in enumerate(rows):
            n = a + i
            data |= {f"A{n}": gen.t4342.TEXTS[key[0]], f"B{n}": f'="{key[1]}"', f"D{n}": i + 1, f"E{n}": 2**i}
            data[f"F{n}"] = f'="{pq[i]}"'
        last = "C" if nf == 3 else "B"
        if nf == 3:
            for i, key in enumerate(rows):
                data[f"C{a + i}"] = gen.t4342.TEXTS[key[2]]
        keys, pq_range = f"Data!A{a}:{last}{b}", f"Data!F{a}:F{b}"
        pos, pw = f"Data!D{a}:D{b}", f"Data!E{a}:E{b}"
        calc[f"A{r}"] = f"{nf}f " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{pq_range},{pos},{agg},0,{nf}),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{pq_range},{pw},{sm},0,{nf},{nf + 1}),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({pq_range},{keys},{pos},{agg},0,0,,{nf}),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({pq_range},{keys},{pos},{agg},0,0,,{nf},-1),1)"
        calc[f"F{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},{agg},0,{nf},-1),1)"
        calc[f"G{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},{agg},0,{nf},-2),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4388: PIVOTBY's and GROUPBY's subtotals on intransitive runs (A = ab, B = ab&U+00AD, C = U+00AD&ab): 30 "
        "seeded five-row runs by p/q rows; PIVOTBY row subtotals (B) and a value sort with them (C), PIVOTBY column "
        "subtotals ascending (D) and col_sort_order -1 (E), GROUPBY subtotals sorted -1 (F) and -2 (G). Written with "
        "the engine's predictions (docs/research/t4388) before the fill.",
        calc,
        data,
    )
    print(r, "blocks")


if __name__ == "__main__":
    main()
