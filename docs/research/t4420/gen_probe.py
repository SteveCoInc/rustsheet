"""T-4420: write probes/t4420-pivotby-parent-sets-three-fields, PIVOTBY's relative_to 3 and 4 on intransitive runs
with three key fields and subtotals at levels 1 and 2 (T-4402's hold-out had two fields per axis only).

Run from the repo root: python3 -I docs/research/t4420/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4420/predict_engine.py, which commits the engine's values as the predictions.

Texts and data columns as docs/research/t4382/gen_probe.py's (A = ab, B = ab&U+00AD, C = U+00AD&ab), from a fresh
seed, leaving out every run of T-4382's and T-4402's probes:
- Calc rows 1-12, by p/q rows with the text, x/y and a second text as column fields and column subtotals at levels
  1 and 2 (col_total_depth 3): B PERCENTOF of 2^(position-1) at the parent column total (relative_to 3), C the
  two-argument LAMBDA showing the cell's records and its set.
- Calc rows 13-24, by the three fields as row fields with row subtotals at levels 1 and 2 (row_total_depth 3), p/q
  columns: B and C as above at the parent row total (relative_to 4).
Every run has a leaf no record reaches or a record in no leaf (t4382's `interesting`).
"""

import importlib.util
import json
import random

_spec = importlib.util.spec_from_file_location("t4382_gen_probe", "docs/research/t4382/gen_probe.py")
assert _spec and _spec.loader
t4382 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4382)
gen = t4382.gen

OUT = "eval/suites/probes/cases/t4420-pivotby-parent-sets-three-fields.case.json"
PREVIOUS = (
    "eval/suites/probes/cases/t4382-pivotby-intransitive-paths.case.json",
    "eval/suites/probes/cases/t4402-pivotby-parent-sets-holdout.case.json",
)


def previous_runs() -> set[str]:
    """The run labels of the earlier probes, which this one leaves out."""
    seen: set[str] = set()
    for path in PREVIOUS:
        with open(path) as fh:
            sheets = json.load(fh)["sheets"]
        calc = next(s["cells"] for s in sheets if s["name"] == "Calc")
        seen |= {str(v).split(" | ")[0] for k, v in calc.items() if k.startswith("A")}
    return seen


def main() -> None:
    rng = random.Random("T-4420")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    seen = previous_runs()
    three = gen.runs(rng, 3)
    row = 1

    def block(rows: list[tuple[str, ...]], pq: list[str]) -> tuple[int, int]:
        nonlocal row
        a = row
        for i, key in enumerate(rows):
            n = a + i
            data.update({f"A{n}": gen.t4342.TEXTS[key[0]], f"B{n}": f'="{key[1]}"', f"D{n}": i + 1, f"E{n}": 11 + i})
            data[f"C{n}"] = gen.t4342.TEXTS[key[2]]
            data[f"F{n}"] = f'="{pq[i]}"'
        row = a + len(rows)
        return a, row - 1

    r = 0
    for last, relative in ((12, 3), (24, 4)):
        while r < last:
            rows = next(three)
            pq = t4382.p_or_q(rng, len(rows))
            name = "piv " + " ".join("".join(k) for k in rows)
            if name in seen or not t4382.interesting(rows):
                continue
            seen.add(name)
            r += 1
            a, b = block(rows, pq)
            pf, keys = f"Data!F{a}:F{b}", f"Data!A{a}:C{b}"
            calc[f"A{r}"] = name + " | " + "".join(pq)
            if relative == 3:
                head, tail = f"{pf},{keys}", "0,0,,3,,,3"
            else:
                head, tail = f"{keys},{pf}", "0,3,,,,,4"
            calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({head},Data!E{a}:E{b},_xleta.PERCENTOF,{tail}),1)"
            calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY({head},Data!D{a}:D{b},{t4382.PAIR},{tail}),1)"
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4420: PIVOTBY's relative_to 3 (rows 1-12) and 4 (rows 13-24) on intransitive runs with three key fields "
        "and subtotals at levels 1 and 2 (A = ab, B = ab&U+00AD, C = U+00AD&ab): PERCENTOF and a two-argument "
        "LAMBDA. Written with the engine's predictions (docs/research/t4420) before the fill.",
        calc,
        data,
    )
    print(r, "blocks,", 2 * r, "formulas")


if __name__ == "__main__":
    main()
