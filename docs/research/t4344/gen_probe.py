"""T-4344: write probes/t4344-groupby-descending-search and its predictions, before the fill.

Run from the repo root: python3 -I docs/research/t4344/gen_probe.py. It writes the case and
docs/research/t4344/hypotheses.tsv: for each GROUPBY block of sections 1 and 3, the ARRAYTOTEXT text each search
hypothesis below predicts. The engine's own predictions for every cell (the current model, T-4342's [0, n-1]
search in the field's direction) are written by `predict_engine.py` into predictions.tsv, also before the fill.

Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B); P = ABC, Q = abc, R = Abc (one class by case).

The hypotheses, each a lower-bound search over [0, n-1] of a parent's classes (docs/research/t4241/model.py,
step 3), differing only in the direction it searches in:
- H0: the field's sort direction (the current model; misses t4342 Calc!C40 only).
- H1: the direction the class list runs in, first class to last (ascending when the last class's key is after the
  first's, descending when before), the field's direction when the two are `=`.
- H2: as H1, ascending when they are `=`.
- H3: the direction of the first two classes.
- H4: the direction of the first class to the first later class not `=` to it.
H1, H3 and H4 give all 434 blocks of t4195, t4241 and t4342 (H0 433: C40; H2 433: C45). They were chosen on those
blocks, so this probe is their hold-out: the blocks of section 1 are picked where they disagree.

Calc rows (one block per row; Data per block row: A the first text, B x/y, C a third text, D the row's position,
E 2^(position-1), F p/q):
- 1-30 (section 1): seeded five-row runs of three fields (A:C), GROUPBY with sort_order -1, -2, -3 or {-1,-3},
  ARRAYTOTEXT of the positions (B). (Of 4000 seeded five-row two-field runs with -1 or -2, the hypotheses agree on
  every one, so section 1 has none.)
- 31-42 (section 2): PIVOTBY with the text as a column field: rows x/y (Data!B), columns the text (Data!A) (B),
  the same with col_sort_order -1 (C); and rows p/q (Data!F), columns text and x/y (Data!A:B) (D), with -1 (E).
- 43-54 (section 3): t4342 Calc!B51:B56's ASCII case-spelling blocks with sort_order -1 (B43:B48) and -2
  (B49:B54).
- 55-66 (section 4): two-field runs with `field_relationship` 1 ascending (B) and -1 (C), and by the SUM of
  2^(position-1) with sort_order 3 (D) and -3 (E).
"""

import importlib.util
import itertools
import json
import random
import sys

sys.path.insert(0, "docs/research/t4241")
# T-4342's generator by path (every probe generator is named gen_probe); loading it patches model.tcmp with the
# P, Q, R class.
_spec = importlib.util.spec_from_file_location("t4342_gen_probe", "docs/research/t4342/gen_probe.py")
assert _spec and _spec.loader
t4342 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4342)

model = t4342.model
OUT = "eval/suites/probes/cases/t4344-groupby-descending-search.case.json"
HYP = "docs/research/t4344/hypotheses.tsv"


def tcmp(a: str, b: str) -> int:
    return model.tcmp(a, b)


def lower_bound(reps: list[str], x: str, d: int) -> int:
    """Lower-bound search over [0, n-1] in direction `d` (T-4342's inclusive search)."""
    lo, hi = 0, len(reps) - 1
    while lo < hi:
        m = (lo + hi) // 2
        if tcmp(reps[m], x) * d < 0:
            lo = m + 1
        else:
            hi = m
    return lo


def h0(reps: list[str], x: str, d: int) -> int:
    return lower_bound(reps, x, d)


def h1(reps: list[str], x: str, d: int) -> int:
    s = tcmp(reps[-1], reps[0])
    return lower_bound(reps, x, s or d)


def h2(reps: list[str], x: str, d: int) -> int:
    s = tcmp(reps[-1], reps[0])
    return lower_bound(reps, x, s or 1)


def h3(reps: list[str], x: str, d: int) -> int:
    s = tcmp(reps[1], reps[0]) if len(reps) > 1 else 0
    return lower_bound(reps, x, s or d)


def h4(reps: list[str], x: str, d: int) -> int:
    for r in reps[1:]:
        s = tcmp(r, reps[0])
        if s:
            return lower_bound(reps, x, s)
    return lower_bound(reps, x, d)


HYPS = {"H0": h0, "H1": h1, "H2": h2, "H3": h3, "H4": h4}


def groupby(rows: list[tuple[str, ...]], dirs: list[int], search) -> list[tuple[tuple[str, ...], list[int]]]:
    """docs/research/t4241/model.py's tree with the search of step 3 replaced by `search`."""
    nf = len(rows[0])
    keys = model.natural_sort(
        [rows[i] for i in model.unique_kept(rows, model.make_rcmp([1] * nf))], model.make_rcmp(dirs)
    )

    def build(ks, f):
        if f == nf:
            return None
        classes: list = []
        for k in ks:
            if classes and tcmp(classes[-1][0][f], k[f]) == 0:
                classes[-1][1].append(k)
            else:
                classes.append((k, [k]))
        return [(c[0][f], build(c[1], f + 1)) for c in classes]

    tree = build(keys, 0)
    leaves: list = []

    def walk(t, path, shown):
        for i, (rep, sub) in enumerate(t):
            if sub is None:
                leaves.append((path + (i,), shown + (rep,)))
            else:
                walk(sub, path + (i,), shown + (rep,))

    walk(tree, (), ())
    members: dict = {}
    for r, row in enumerate(rows, 1):
        t, path = tree, ()
        for f in range(nf):
            reps = [x[0] for x in t]
            i = search(reps, row[f], dirs[f])
            if tcmp(reps[i], row[f]) != 0:
                path = None
                break
            path, t = path + (i,), t[i][1]
        if path is not None:
            members.setdefault(path, []).append(r)
    return [(shown, members.get(p) or list(range(1, len(rows) + 1))) for p, shown in leaves]


def text(groups) -> str:
    """The ARRAYTOTEXT (strict) text of a GROUPBY of positions with an ARRAYTOTEXT aggregate."""
    out = []
    for key, ms in groups:
        out.append(
            ",".join(
                [
                    *(json.dumps(t4342.SHOW.get(k, k), ensure_ascii=False) for k in key),
                    json.dumps(", ".join(map(str, ms))),
                ]
            )
        )
    return "{" + ";".join(out) + "}"


def runs(rng: random.Random, fields: int, texts: str = "ABC"):
    """Endless seeded five-row runs (each text present, both x and y)."""
    while True:
        first = [rng.choice(texts) for _ in range(5)]
        if set(first) != set(texts):
            continue
        xy = [rng.choice("xy") for _ in range(5)]
        if len(set(xy)) < 2:
            continue
        cols = [first, xy] + ([[rng.choice(texts) for _ in range(5)]] if fields == 3 else [])
        yield [tuple(c[i] for c in cols) for i in range(5)]


def dirs_of(order: str, nf: int) -> list[int]:
    dirs = [1] * nf
    for o in order.strip("{}").split(","):
        if int(o) < 0:
            dirs[-int(o) - 1] = -1
    return dirs


def discriminating(rng: random.Random, fields: int, orders: list[str], count: int):
    """`count` distinct (rows, order) blocks where the hypotheses disagree, drawn from 4000 seeded candidates
    round-robin over the disagreement patterns (which hypotheses agree with which), so each pattern is shown."""
    pool: dict[tuple, list] = {}
    seen = set()
    gen = runs(rng, fields)
    for _ in range(4000):
        rows, order = next(gen), rng.choice(orders)
        if (tuple(rows), order) in seen:
            continue
        seen.add((tuple(rows), order))
        preds = {k: text(groupby(rows, dirs_of(order, fields), h)) for k, h in HYPS.items()}
        pattern = tuple(sorted({tuple(k for k in HYPS if preds[k] == v) for v in preds.values()}))
        if len(pattern) > 1:
            pool.setdefault(pattern, []).append((rows, order, preds))
    out: list = []
    queues = [pool[p] for p in sorted(pool)]
    while len(out) < count and any(queues):
        for q in queues:
            if q and len(out) < count:
                out.append(q.pop(0))
    return out


def main() -> None:
    rng = random.Random("T-4344")
    data: dict[str, object] = {}
    calc: dict[str, str] = {}
    hyps: list[tuple[str, dict[str, str]]] = []
    row = 1
    r = 0

    def put(rows, pq=None):
        nonlocal row
        a, b = row, row + len(rows) - 1
        if pq is None:
            pq = [rng.choice("pq") for _ in rows]
            if len(set(pq)) < 2:
                pq[0] = "q" if pq[0] == "p" else "p"
        for i, key in enumerate(rows):
            n = row + i
            data[f"A{n}"] = t4342.TEXTS[key[0]]
            data[f"B{n}"] = f'="{key[1]}"'
            if len(key) == 3:
                data[f"C{n}"] = t4342.TEXTS[key[2]]
            data[f"D{n}"] = i + 1
            data[f"E{n}"] = 2**i
            data[f"F{n}"] = f'="{pq[i]}"'
        row = b + 1
        return a, b, pq

    def label(kind, rows, pq, extra=""):
        return f"{kind} " + " ".join("".join(k) for k in rows) + " | " + "".join(pq) + extra

    agg = "LAMBDA(v,ARRAYTOTEXT(v))"
    # Section 1: descending searches.
    blocks = discriminating(rng, 3, ["-1", "-2", "-3", "{-1,-3}"], 30)
    for rows, order, preds in blocks:
        r += 1
        a, b, pq = put(rows)
        last = "C" if len(rows[0]) == 3 else "B"
        calc[f"A{r}"] = label(f"{len(rows[0])}f", rows, pq, f" | {order}")
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY(Data!A{a}:{last}{b},Data!D{a}:D{b},{agg},0,0,{order}),1)"
        hyps.append((f"B{r}", preds))
    # Section 2: PIVOTBY with the intransitive text as a column field.
    for rows in itertools.islice(runs(rng, 2), 12):
        r += 1
        a, b, pq = put(rows)
        calc[f"A{r}"] = label("piv", rows, pq)
        pos = f"Data!D{a}:D{b}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!B{a}:B{b},Data!A{a}:A{b},{pos},{agg},0,0,,0),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!B{a}:B{b},Data!A{a}:A{b},{pos},{agg},0,0,,0,-1),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!F{a}:F{b},Data!A{a}:B{b},{pos},{agg},0,0,,0),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.PIVOTBY(Data!F{a}:F{b},Data!A{a}:B{b},{pos},{agg},0,0,,0,-1),1)"
    # Section 3: ASCII case spellings, descending.
    case_blocks = [
        [("P", "y"), ("Q", "x")],
        [("Q", "y"), ("P", "x"), ("R", "x")],
        [("P", "y"), ("Q", "x"), ("R", "y")],
        [("R", "y"), ("P", "y"), ("Q", "x")],
        [("P", "x"), ("Q", "y")],
        [("Q", "y"), ("R", "x"), ("P", "y"), ("Q", "x")],
    ]
    for order in ("-1", "-2"):
        for rows in case_blocks:
            r += 1
            a, b, pq = put(rows, ["p"] * len(rows))
            calc[f"A{r}"] = label("case", rows, pq, f" | {order}")
            calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY(Data!A{a}:B{b},Data!D{a}:D{b},{agg},0,0,{order}),1)"
            hyps.append((f"B{r}", {k: text(groupby(rows, dirs_of(order, 2), h)) for k, h in HYPS.items()}))
    # Section 4: table relationship and value sorts.
    for rows in itertools.islice(runs(rng, 2), 12):
        r += 1
        a, b, pq = put(rows)
        calc[f"A{r}"] = label("tbl", rows, pq)
        keys, pos, pw = f"Data!A{a}:B{b}", f"Data!D{a}:D{b}", f"Data!E{a}:E{b}"
        calc[f"B{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},{agg},0,0,1,,1),1)"
        calc[f"C{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},{agg},0,0,-1,,1),1)"
        calc[f"D{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},LAMBDA(v,SUM(v)),0,0,3),1)"
        calc[f"E{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},LAMBDA(v,SUM(v)),0,0,-3),1)"
    model_probe = t4342.t4241_probe
    model_probe.write_case(
        OUT,
        "T-4344: a hold-out for GROUPBY's tree search on intransitive runs (A = ab, B = ab&U+00AD, "
        "C = U+00AD&ab; A = C = B, A < B) under descending sorts, with PIVOTBY column fields, ASCII case spellings "
        "descending, a table relationship and value sorts. Rows 1-30: seeded five-row three-field runs picked where "
        "the search hypotheses of docs/research/t4344 disagree; 31-42: PIVOTBY by the text as a column field; 43-54: "
        "case spellings with sort_order -1 and -2; 55-66: field_relationship 1 and sort by a SUM. Written with the "
        "predictions (docs/research/t4344) before the fill.",
        calc,
        data,
    )
    with open(HYP, "w") as fh:
        fh.write(
            "# T-4344: each search hypothesis's ARRAYTOTEXT for probes/t4344-groupby-descending-search, written "
            "before the oracle fill (cell, then one column per hypothesis: " + ", ".join(HYPS) + ")\n"
        )
        for c, preds in hyps:
            fh.write(c + "\t" + "\t".join(preds[k] for k in HYPS) + "\n")
    print(r, "blocks,", len(calc) - r, "formulas,", len(hyps), "hypothesis lines")


if __name__ == "__main__":
    main()
