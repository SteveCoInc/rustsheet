"""T-4402: write probes/t4402-pivotby-parent-set-holdout, the hold-out of PIVOTBY's relative_to 3 and 4 parent sets
on intransitive runs (T-4382's Calc!C1:E30): the parent set is the parent subtotal's records (those that reached
its class, every record of the line when none did), and an empty parent set is PERCENTOF `#DIV/0!` and a
two-argument LAMBDA `#VALUE!`.

Run from the repo root: python3 -I docs/research/t4402/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4402/predict_engine.py, which commits the engine's values as the predictions.

Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B). Data per block row: A the text, B x/y, D the
row's position, E 2^(position-1), F p/q. Fresh seeded five-row runs (seed "T-4402"), two key fields (text, x/y)
on one axis and p/q on the other, ascending, subtotals of the first field:
- Calc rows 1-12: an empty parent (a leaf no record reaches under a class that records reached, none of them of
  one p/q group: that group's cell under the leaf has records, its parent none);
- rows 13-24: any other run with a leaf no record reaches or a record in no leaf.
Columns: B PERCENTOF with the keys as column fields (relative_to 3, the parent column total), C the two-argument
LAMBDA there, D PERCENTOF with the keys as row fields (relative_to 4, the parent row total), E the LAMBDA there.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

OUT = "eval/suites/probes/cases/t4402-pivotby-parent-set-holdout.case.json"
PAIR = 'LAMBDA(x,a,ARRAYTOTEXT(x)&" / "&ARRAYTOTEXT(a))'
BLOCKS = 12


def empty_parent(rows: list[tuple[str, ...]], pq: list[str]) -> bool:
    """Whether a leaf no record reaches sits under a first-field class that records reached, none of one p/q
    group's."""
    leaves, paths = gen.descents(rows, [1, 1], gen.h3)
    reached = set(paths)
    for path, _ in leaves:
        if path in reached:
            continue
        under = [i for i, p in enumerate(paths) if p[:1] == path[:1]]
        if under and any(all(pq[i] != g for i in under) for g in "pq"):
            return True
    return False


def interesting(rows: list[tuple[str, ...]]) -> bool:
    """A leaf no record reaches or a record in no leaf (ascending)."""
    return sum(len(ms) for _, ms in gen.groupby(rows, [1, 1], gen.h3)) != len(rows)


def main() -> None:
    rng = random.Random("T-4402")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    source = gen.runs(rng, 2)
    row = 1
    for r in range(1, 2 * BLOCKS + 1):
        while True:
            rows = next(source)
            pq = [rng.choice("pq") for _ in rows]
            if tuple(rows) in seen or len(set(pq)) < 2 or not interesting(rows):
                continue
            if empty_parent(rows, pq) == (r <= BLOCKS):
                break
        seen.add(tuple(rows))
        a = row
        for i, key in enumerate(rows):
            n = a + i
            data[f"A{n}"] = gen.t4342.TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            data[f"D{n}"] = i + 1
            data[f"E{n}"] = 2**i
            data[f"F{n}"] = f'="{pq[i]}"'
        row = a + len(rows)
        b = row - 1
        keys, pqs, pos, pow2 = f"Data!A{a}:B{b}", f"Data!F{a}:F{b}", f"Data!D{a}:D{b}", f"Data!E{a}:E{b}"
        calc[f"A{r}"] = "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({pqs},{keys},{pow2},_xleta.PERCENTOF,0,0,,2,,,3),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({pqs},{keys},{pos},{PAIR},0,0,,2,,,3),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{pqs},{pow2},_xleta.PERCENTOF,0,2,,,,,4),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{pqs},{pos},{PAIR},0,2,,,,,4),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4402: hold-out of PIVOTBY's relative_to 3 and 4 parent sets on intransitive runs (A = ab, "
        "B = ab&U+00AD, C = U+00AD&ab): fresh seeded runs with an empty parent set (rows 1-12) and others with a "
        "leaf no record reaches or a record in no leaf (13-24); PERCENTOF and a two-argument LAMBDA at the parent "
        "column total (B, C) and the parent row total (D, E). Written with the engine's predictions "
        "(docs/research/t4402) before the fill.",
        calc,
        data,
    )
    print(2 * BLOCKS, "blocks,", len(calc) - 2 * BLOCKS, "formulas")


if __name__ == "__main__":
    main()
