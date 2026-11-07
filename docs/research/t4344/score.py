"""T-4344: score the search hypotheses of gen_probe.py and the engine's predictions.

Run from the repo root: python3 -I docs/research/t4344/score.py. Prints each hypothesis's score on the 434 earlier
blocks (t4195 C and E, t4241 B to E, t4342 B to E of the A/B/C runs; H0 433, H1 434, H2 433, H3 434, H4 434), on
this probe's 42 hypothesis cells (H3 42), and the engine's committed predictions against the fill, with each miss.
"""

import importlib.util
import json

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)
model = gen.model
_cspec = importlib.util.spec_from_file_location("t4342_compare", "docs/research/t4342/compare.py")
assert _cspec and _cspec.loader
compare = importlib.util.module_from_spec(_cspec)
_cspec.loader.exec_module(compare)

CASE = "eval/suites/probes/cases/t4344-groupby-descending-search"


def earlier(search) -> tuple[int, int, list[str]]:
    """The hypothesis's score on the earlier probes' A/B/C blocks."""
    ok, total, bad = 0, 0, []
    for case, col, nf in [
        ("t4195-groupby-intransitive", "C", 2),
        ("t4195-groupby-intransitive", "E", 3),
        ("t4241-groupby-intransitive-runs", "B", 2),
        ("t4241-groupby-intransitive-runs", "C", 2),
        ("t4241-groupby-intransitive-runs", "D", 2),
        ("t4241-groupby-intransitive-runs", "E", 2),
    ]:
        for cell, rows, want, formula in model.load(case, col, nf):
            total += 1
            if gen.groupby(rows, model.sort_dirs(formula, nf), search) == want:
                ok += 1
            else:
                bad.append(f"{case} {cell}")
    with open(compare.CASE + ".case.json") as fh:
        calc = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["Calc"]
    with open(compare.CASE + ".oracle.json") as fh:
        oracle = json.load(fh)["sheets"]["Calc"]
    for cell in sorted(c for c in calc if c[0] in "BCDE" and str(calc[c]).startswith("=")):
        label = calc[f"A{cell[1:]}"]
        if label.startswith("case"):
            continue
        keys, pq = label.split("|")
        rows = [tuple(k) for k in keys.split()[1:]]
        nf = len(rows[0])
        dirs = [-1] + [1] * (nf - 1) if cell[0] == "C" else [1] * nf
        groups = gen.groupby(rows, dirs, search)
        if cell[0] == "E":
            cols = sorted(set(pq.strip()))
            want = json.dumps(
                {
                    "columns": cols,
                    "rows": [
                        [*map(t4342_show, k), *[[m for m in ms if pq.strip()[m - 1] == c] for c in cols]]
                        for k, ms in groups
                    ],
                },
                ensure_ascii=False,
            )
        elif cell[0] == "D":
            want = json.dumps([[*map(t4342_show, k), sum(2 ** (m - 1) for m in ms)] for k, ms in groups],
                              ensure_ascii=False)
        else:
            want = json.dumps([[*map(t4342_show, k), ms] for k, ms in groups], ensure_ascii=False)
        total += 1
        if compare.canonical(cell[0], oracle[cell]["v"], nf) == want:
            ok += 1
        else:
            bad.append(f"t4342 {cell}")
    return ok, total, bad


def t4342_show(k: str) -> str:
    return gen.t4342.SHOW.get(k, k)


def main() -> None:
    with open(CASE + ".oracle.json") as fh:
        oracle = json.load(fh)["sheets"]["Calc"]
    names = list(gen.HYPS)
    for name, search in gen.HYPS.items():
        ok, total, bad = earlier(search)
        print(f"{name} earlier blocks: {ok} / {total}", bad[:4])
    scores = {n: 0 for n in names}
    lines = 0
    with open(gen.HYP) as fh:
        for line in fh:
            if line.startswith("#"):
                continue
            cell, *preds = line.rstrip("\n").split("\t")
            lines += 1
            for n, p in zip(names, preds):
                scores[n] += oracle[cell]["v"] == p
    print("hold-out:", {n: f"{s} / {lines}" for n, s in scores.items()})
    misses = []
    with open("docs/research/t4344/predictions.tsv") as fh:
        for line in fh:
            if line.startswith("#"):
                continue
            cell, value = line.rstrip("\n").split("\t", 1)
            if json.loads(value) != oracle[cell]["v"]:
                misses.append(cell)
    print(f"engine before T-4344 (H0): {len(oracle) - len(misses)} / {len(oracle)}; misses", misses)


if __name__ == "__main__":
    main()
