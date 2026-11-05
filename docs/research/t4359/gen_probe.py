"""T-4359: write probes/t4359-groupby-value-sort-parents, a hold-out for a value sort's parent values.

Run from the repo root: python3 -I docs/research/t4359/gen_probe.py, then (after `make build`, before the fill)
python3 -I docs/research/t4359/predict_engine.py. This writes the case and docs/research/t4359/hypotheses.tsv: for
each B, C and D cell, the groups (shown keys and value, in order) each hypothesis of model.py predicts.

The rule under test, R1 (chosen on probes/t4344-groupby-descending-search Calc!D56, E56): a parent's value is the
function of the rows that reached its class, those that reached no leaf under it included. Blocks are seeded
five-row runs (seed "T-4359", fresh) picked where R0, R1 and R2 order the groups differently in B, C or D,
round-robin over the disagreement patterns: 14 of two fields (text, x/y) and 14 of three (text, x/y, text), so
rows also drop at the second field of three. Data per block row: A the text, B x/y, C the third text, D the row's
position, E 2^(position-1). Calc columns (sort_order names the value column, nf + 1):
- B: SUM of E, ascending, total_depth 0;
- C: the same descending;
- D: AVERAGE of E, ascending, total_depth 0 (the dropped row moves an average);
- E: SUM of E ascending with subtotals at every level (total_depth nf): which rows a shown subtotal and the grand
  total hold (exploratory: the engine's values are its predictions).
"""

import importlib.util
import json
import random

_spec = importlib.util.spec_from_file_location("t4359_model", "docs/research/t4359/model.py")
assert _spec and _spec.loader
model = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(model)
gen = model.gen

OUT = "eval/suites/probes/cases/t4359-groupby-value-sort-parents.case.json"
HYP = "docs/research/t4359/hypotheses.tsv"
HYPS = ("R0", "R1", "R2")
PER = 14


def total(rs: list[int]) -> float:
    return float(sum(2 ** (r - 1) for r in rs))


def average(rs: list[int]) -> float:
    return total(rs) / len(rs) if rs else 0.0


COLUMNS = {"B": (total, True), "C": (total, False), "D": (average, True)}


def predict(rows, column: str, hyp: str) -> list:
    agg, ascending = COLUMNS[column]
    groups, _ = model.value_sort(rows, agg, ascending, hyp)
    return [[*(gen.t4342.SHOW.get(k, k) for k in shown), value] for shown, value in groups]


def pick(rng: random.Random, fields: int, count: int, seen: set) -> list:
    pool: dict[tuple, list] = {}
    runs = gen.runs(rng, fields)
    for _ in range(4000):
        rows = next(runs)
        if tuple(rows) in seen:
            continue
        seen.add(tuple(rows))
        preds = {c: {h: predict(rows, c, h) for h in HYPS} for c in COLUMNS}
        pattern = tuple(
            tuple(sorted({tuple(h for h in HYPS if preds[c][h] == v) for v in preds[c].values()})) for c in COLUMNS
        )
        if any(len(p) > 1 for p in pattern):
            pool.setdefault(pattern, []).append((rows, preds))
    out: list = []
    queues = [pool[p] for p in sorted(pool)]
    while len(out) < count and any(queues):
        for q in queues:
            if q and len(out) < count:
                out.append(q.pop(0))
    return out


def main() -> None:
    rng = random.Random("T-4359")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    hyps: list[tuple[str, dict[str, list]]] = []
    seen: set = set()
    blocks = pick(rng, 2, PER, seen) + pick(rng, 3, PER, seen)
    for r, (rows, preds) in enumerate(blocks, start=1):
        a, b = 5 * r - 4, 5 * r
        nf = len(rows[0])
        for i, key in enumerate(rows):
            n = a + i
            data[f"A{n}"] = gen.t4342.TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            if nf == 3:
                data[f"C{n}"] = gen.t4342.TEXTS[key[2]]
            data[f"D{n}"] = i + 1
            data[f"E{n}"] = 2**i
        last = "C" if nf == 3 else "B"
        keys, pw = f"Data!A{a}:{last}{b}", f"Data!E{a}:E{b}"
        calc[f"A{r}"] = f"{nf}f " + " ".join("".join(k) for k in rows)
        sm, av = "LAMBDA(v,SUM(v))", "LAMBDA(v,AVERAGE(v))"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,0,{nf + 1}),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,0,{-(nf + 1)}),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{av},0,0,{nf + 1}),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},{sm},0,{nf},{nf + 1}),1)"
        for c in COLUMNS:
            hyps.append((f"{c}{r}", preds[c]))
    gen.t4342.t4241_probe.write_case(
        OUT,
        "T-4359: a hold-out for GROUPBY's sort by a value on intransitive runs (A = ab, B = ab&U+00AD, "
        "C = U+00AD&ab; A = C = B, A < B): 14 seeded five-row two-field and 14 three-field runs picked where the "
        "parent-value hypotheses of docs/research/t4359 order the groups differently; SUM of 2^(position-1) "
        "ascending (B) and descending (C), AVERAGE ascending (D), SUM with subtotals at every level (E). Written "
        "with the predictions (docs/research/t4359) before the fill.",
        calc,
        data,
    )
    with open(HYP, "w") as fh:
        fh.write(
            "# T-4359: each hypothesis's groups (shown keys, value, in order) for probes/t4359-groupby-value-sort-"
            "parents, written before the oracle fill (cell, then one JSON column per hypothesis: "
            + ", ".join(HYPS)
            + ")\n"
        )
        for c, preds in hyps:
            fh.write(c + "\t" + "\t".join(json.dumps(preds[h], ensure_ascii=False) for h in HYPS) + "\n")
    print(len(blocks), "blocks,", len(hyps), "hypothesis lines")


if __name__ == "__main__":
    main()
