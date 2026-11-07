"""T-4382: write probes/t4382-pivotby-intransitive-paths, PIVOTBY's paths on intransitive runs that T-4378's
subtotal rule was not checked on.

Run from the repo root: python3 -I docs/research/t4382/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4382/predict_engine.py, which commits the engine's values as the predictions.

Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B). Data per block row: A the text, B x/y, C a
second text (three-field blocks), D the row's position, E 10 + the position, F p/q. Seeded five-row runs:
- Calc rows 1-15, by p/q rows with the text and x/y as column fields and column subtotals (col_total_depth 2):
  - B: the subtotal error gate. One function over two value columns (D:E), an ARRAYTOTEXT that is NA() on the
    record set `target` (the Calc A label after "!"). Each run has a first-field subtotal whose records reached its
    class but no leaf under it, with every leaf under it empty for that row group (`target` is that subtotal's
    records), or an empty subtotal with a leaf no record reaches under it (`target` none), where the engine's gate
    (`Grid::has_records`, the union of the leaf buckets) and the subtotal's records (`Grid::cell_rows`) disagree.
  - C: PERCENTOF of 2^(position-1) relative to the parent column total (relative_to 3).
  - D: a two-argument LAMBDA showing the cell's records and its parent column total's (relative_to 3).
- Calc rows 16-30, by the text and x/y as row fields with row subtotals (row_total_depth 2), p/q columns, runs whose
  tree has a leaf no record reaches or a record that reaches a class but no leaf:
  - B: ARRAYTOTEXT of the positions, ascending; C: the same with row_sort_order -1;
  - D: PERCENTOF relative to the parent row total (relative_to 4); E: the two-argument LAMBDA at relative_to 4.
- Calc rows 31-42, three column fields (text, x/y, text) with subtotals at levels 1 and 2 (col_total_depth 3) by
  p/q rows, runs of the same kind: B ascending, C col_sort_order -1.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4358_gen_probe", "docs/research/t4358/gen_probe.py")
assert _spec and _spec.loader
t4358 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4358)
_spec = importlib.util.spec_from_file_location("t4358_score", "docs/research/t4358/score.py")
assert _spec and _spec.loader
score = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(score)
gen = t4358.gen

OUT = "eval/suites/probes/cases/t4382-pivotby-intransitive-paths.case.json"
SHOW = "LAMBDA(v,ARRAYTOTEXT(v))"
PAIR = 'LAMBDA(x,a,ARRAYTOTEXT(x)&" / "&ARRAYTOTEXT(a))'


def gate_target(rows: list[tuple[str, ...]], pq: list[str]) -> str | None:
    """The run's gate case: the records of the first subtotal (ascending column tree) that has records for a row
    group while every leaf under it is empty for it, "none" when there is no such subtotal but one that is empty
    for a row group has a non-empty leaf (a leaf no record reaches), or None when neither occurs."""
    reps, found = score.first_classes(rows)
    leaves = gen.groupby(rows, [1, 1], gen.h3)
    classes: list[list[list[int]]] = []
    for i, (shown, members) in enumerate(leaves):
        if i == 0 or gen.tcmp(leaves[i - 1][0][0], shown[0]) != 0:
            classes.append([])
        classes[-1].append(members)
    assert len(classes) == len(reps)
    empty_with_leaf = False
    for group in "pq":
        mine = {i + 1 for i in range(len(rows)) if pq[i] == group}
        for k, leaf_members in enumerate(classes):
            reached = [i + 1 for i in range(len(rows)) if pq[i] == group and found[i] == k]
            held = set().union(*(set(m) & mine for m in leaf_members))
            if reached and not held:
                return ", ".join(map(str, reached))
            empty_with_leaf |= not reached and bool(held)
    return "none" if empty_with_leaf else None


def interesting(rows: list[tuple[str, ...]]) -> bool:
    """A leaf no record reaches or a record in no leaf, ascending or with the first field descending."""
    n = len(rows[0])
    for first in (1, -1):
        dirs = [first] + [1] * (n - 1)
        groups = gen.groupby(rows, dirs, gen.h3)
        if sum(len(ms) for _, ms in groups) != len(rows):
            return True
    return False


def p_or_q(rng: random.Random, n: int) -> list[str]:
    pq = [rng.choice("pq") for _ in range(n)]
    if len(set(pq)) < 2:
        pq[0] = "q" if pq[0] == "p" else "p"
    return pq


def main() -> None:
    rng = random.Random("T-4382")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen: set = set()
    two, three = gen.runs(rng, 2), gen.runs(rng, 3)
    row = 1

    def block(rows: list[tuple[str, ...]], pq: list[str]) -> tuple[int, int]:
        nonlocal row
        a = row
        for i, key in enumerate(rows):
            n = a + i
            data.update({f"A{n}": gen.t4342.TEXTS[key[0]], f"B{n}": f'="{key[1]}"', f"D{n}": i + 1, f"E{n}": 11 + i})
            data[f"F{n}"] = f'="{pq[i]}"'
            if len(key) > 2:
                data[f"C{n}"] = gen.t4342.TEXTS[key[2]]
        row = a + len(rows)
        return a, row - 1

    def label(rows: list[tuple[str, ...]], pq: list[str]) -> str:
        return "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)

    r = 0
    while r < 15:
        rows = next(two)
        pq = p_or_q(rng, len(rows))
        target = gate_target(rows, pq)
        if tuple(rows) in seen or target is None:
            continue
        seen.add(tuple(rows))
        r += 1
        a, b = block(rows, pq)
        rf, cols = f"Data!F{a}:F{b}", f"Data!A{a}:B{b}"
        calc[f"A{r}"] = label(rows, pq) + " ! " + target
        gate = f'LAMBDA(v,IF(ARRAYTOTEXT(v)="{target}",NA(),ARRAYTOTEXT(v)))'
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},Data!D{a}:E{b},{gate},0,0,,2),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},Data!E{a}:E{b},_xleta.PERCENTOF,0,0,,2,,,3),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},Data!D{a}:D{b},{PAIR},0,0,,2,,,3),1)"
    while r < 30:
        rows = next(two)
        if tuple(rows) in seen or not interesting(rows):
            continue
        seen.add(tuple(rows))
        pq = p_or_q(rng, len(rows))
        r += 1
        a, b = block(rows, pq)
        keys, cf, pos = f"Data!A{a}:B{b}", f"Data!F{a}:F{b}", f"Data!D{a}:D{b}"
        calc[f"A{r}"] = label(rows, pq)
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{cf},{pos},{SHOW},0,2),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{cf},{pos},{SHOW},0,2,-1),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{cf},Data!E{a}:E{b},_xleta.PERCENTOF,0,2,,,,,4),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({keys},{cf},{pos},{PAIR},0,2,,,,,4),1)"
    while r < 42:
        rows = next(three)
        if tuple(rows) in seen or not interesting(rows):
            continue
        seen.add(tuple(rows))
        pq = p_or_q(rng, len(rows))
        r += 1
        a, b = block(rows, pq)
        rf, cols, pos = f"Data!F{a}:F{b}", f"Data!A{a}:C{b}", f"Data!D{a}:D{b}"
        calc[f"A{r}"] = label(rows, pq)
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{SHOW},0,0,,3),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({rf},{cols},{pos},{SHOW},0,0,,3,-1),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4382: PIVOTBY on intransitive runs (A = ab, B = ab&U+00AD, C = U+00AD&ab) where T-4378's subtotal rule "
        "was not checked: the subtotal error gate of one function over two value columns and PERCENTOF / a "
        "two-argument LAMBDA at the parent column total (rows 1-15), row-axis subtotals ascending and descending and "
        "the parent row total (16-30), and three column fields with subtotals at levels 1 and 2 (31-42). Written "
        "with the engine's predictions (docs/research/t4382) before the fill.",
        calc,
        data,
    )
    print(r, "blocks,", len(calc) - r, "formulas")


if __name__ == "__main__":
    main()
