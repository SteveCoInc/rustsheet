"""T-4378: write probes/t4378-pivotby-subtotal-holdout, a hold-out for PIVOTBY's column subtotals on intransitive runs.

Run from the repo root: python3 -I docs/research/t4378/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4378/predict_engine.py, which commits the engine's values as the predictions.

The rule under test (chosen on probes/t4358-pivotby-unreached-column-leaf column D): a first-field subtotal column
holds the row group's records whose descent reached that class, those that reached no leaf under it included, and
not the extra records of a leaf no record reaches. 25 fresh seeded five-row runs (seed "T-4378", so none of t4358's)
whose column tree has a leaf no record reaches or a record that reaches a class but no leaf, by p/q rows; columns
the text and x/y with column subtotals: ascending (B), `col_sort_order` -1 (C), and by SUM of 2^(position-1) (D).
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location(
    "t4358_gen_probe", "docs/research/t4358/gen_probe.py"
)
assert _spec and _spec.loader
t4358 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4358)
gen = t4358.gen

OUT = "eval/suites/probes/cases/t4378-pivotby-subtotal-holdout.case.json"


def interesting(rows: list[tuple[str, ...]]) -> bool:
    """A leaf no record reaches, or a record in no leaf (the groups hold fewer rows than the run)."""
    for dirs in ([1, 1], [-1, 1]):
        held = {m for _, ms in gen.groupby(rows, dirs, gen.h3) for m in ms}
        if t4358.has_unreached(rows, dirs) or len(held) < len(rows):
            return True
    return False


def main() -> None:
    rng = random.Random("T-4378")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    runs = gen.runs(rng, 2)
    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    r = 0
    while r < 25:
        rows = next(runs)
        if tuple(rows) in seen or not interesting(rows):
            continue
        seen.add(tuple(rows))
        pq = [rng.choice("pq") for _ in rows]
        if len(set(pq)) < 2:
            pq[0] = "q" if pq[0] == "p" else "p"
        a = 5 * r + 1
        r += 1
        for i, key in enumerate(rows):
            n = a + i
            data |= {
                f"A{n}": gen.t4342.TEXTS[key[0]],
                f"B{n}": f'="{key[1]}"',
                f"D{n}": i + 1,
                f"E{n}": 2**i,
            }
            data[f"F{n}"] = f'="{pq[i]}"'
        b = a + 4
        calc[f"A{r}"] = (
            "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
        )
        head = f"_xlfn.PIVOTBY(Data!F{a}:F{b},Data!A{a}:B{b}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT({head},Data!D{a}:D{b},{agg},0,0,,2),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT({head},Data!D{a}:D{b},{agg},0,0,,2,-1),1)"
        calc[f"D{r}"] = (
            f"=ARRAYTOTEXT({head},Data!E{a}:E{b},LAMBDA(v,SUM(v)),0,0,,2),1)"
        )
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4378: a hold-out for PIVOTBY's column subtotals on intransitive runs (A = ab, B = ab&U+00AD, "
        "C = U+00AD&ab): 25 seeded five-row runs by p/q rows with column subtotals, ascending (B), col_sort_order -1 "
        "(C) and as SUM (D). Written with the engine's predictions (docs/research/t4378) before the fill.",
        calc,
        data,
    )
    print(r, "blocks")


if __name__ == "__main__":
    main()
