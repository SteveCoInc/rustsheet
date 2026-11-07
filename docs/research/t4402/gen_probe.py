"""T-4402: write probes/t4402-pivotby-parent-sets-holdout, a hold-out for PIVOTBY's relative_to 3 and 4 on
intransitive runs (the rule T-4382's cells chose: a cell's parent set is the records of its parent subtotal's
cell, read by the subtotal rule, and an empty one is an error).

Run from the repo root: python3 -I docs/research/t4402/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4402/predict_engine.py, which commits the engine's values as the predictions.

Texts and data columns as docs/research/t4382/gen_probe.py's, from a fresh seed (no run of that probe repeats):
- Calc rows 1-16, by p/q rows with the text and x/y as column fields and column subtotals (col_total_depth 2):
  B PERCENTOF of 2^(position-1) at the parent column total (relative_to 3), C the two-argument LAMBDA showing the
  cell's records and its set; the first 8 runs have a row group no record of which reached a first-field class
  that holds a leaf no record reaches (an empty parent set), the rest any run with such a leaf or a record in no
  leaf.
- Calc rows 17-32, by the text and x/y as row fields with row subtotals (row_total_depth 2), p/q columns: B and C
  as above at the parent row total (relative_to 4), chosen the same way with the axes swapped.
"""

import importlib.util
import random

_spec = importlib.util.spec_from_file_location("t4382_gen_probe", "docs/research/t4382/gen_probe.py")
assert _spec and _spec.loader
t4382 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4382)
gen = t4382.gen

OUT = "eval/suites/probes/cases/t4402-pivotby-parent-sets-holdout.case.json"
PREVIOUS = "eval/suites/probes/cases/t4382-pivotby-intransitive-paths.case.json"


def previous_runs() -> set[str]:
    """The run labels of T-4382's probe, which the hold-out leaves out."""
    import json

    with open(PREVIOUS) as fh:
        sheets = json.load(fh)["sheets"]
    calc = next(s["cells"] for s in sheets if s["name"] == "Calc")
    return {str(v).split(" | ")[0] for k, v in calc.items() if k.startswith("A")}


def main() -> None:
    rng = random.Random("T-4402")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen = previous_runs()
    two = gen.runs(rng, 2)
    row = 1

    def block(rows: list[tuple[str, ...]], pq: list[str]) -> tuple[int, int]:
        nonlocal row
        a = row
        for i, key in enumerate(rows):
            n = a + i
            data.update({f"A{n}": gen.t4342.TEXTS[key[0]], f"B{n}": f'="{key[1]}"', f"D{n}": i + 1, f"E{n}": 11 + i})
            data[f"F{n}"] = f'="{pq[i]}"'
        row = a + len(rows)
        return a, row - 1

    def label(rows: list[tuple[str, ...]], pq: list[str]) -> str:
        return "piv " + " ".join("".join(k) for k in rows) + " | " + "".join(pq)

    r = 0
    for first, last, relative in ((1, 16, 3), (17, 32, 4)):
        while r < last:
            rows = next(two)
            pq = t4382.p_or_q(rng, len(rows))
            name = label(rows, pq).split(" | ")[0]
            empty = t4382.gate_target(rows, pq) == "none"
            wanted = empty if r < first + 7 else t4382.interesting(rows)
            if name in seen or not wanted:
                continue
            seen.add(name)
            r += 1
            a, b = block(rows, pq)
            pf, keys = f"Data!F{a}:F{b}", f"Data!A{a}:B{b}"
            calc[f"A{r}"] = label(rows, pq) + (" ! empty" if empty else "")
            if relative == 3:
                head = f"{pf},{keys}"
                tail = "0,0,,2,,,3"
            else:
                head = f"{keys},{pf}"
                tail = "0,2,,,,,4"
            calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({head},Data!E{a}:E{b},_xleta.PERCENTOF,{tail}),1)"
            calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({head},Data!D{a}:D{b},{t4382.PAIR},{tail}),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4402: hold-out for PIVOTBY's relative_to 3 (rows 1-16) and 4 (rows 17-32) on intransitive runs (A = ab, "
        "B = ab&U+00AD, C = U+00AD&ab): PERCENTOF and a two-argument LAMBDA, runs with an empty parent set first. "
        "Written with the engine's predictions (docs/research/t4402) before the fill.",
        calc,
        data,
    )
    print(r, "blocks,", 2 * r, "formulas")


if __name__ == "__main__":
    main()
