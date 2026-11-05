"""T-4342: score docs/research/t4342/predictions.tsv against the oracle of probes/t4342-groupby-tree-holdout.

Run from the repo root: python3 -I docs/research/t4342/compare.py [--revised]. Reads each Calc cell's ARRAYTOTEXT
(strict) text into the canonical form gen_probe.py wrote its predictions in, and prints the hits and every miss.
Without --revised it scores the frozen predictions (199 of 206); with it, the model with T-4342's inclusive
search, recomputed from each block's label (205 of 206).
"""

import json
import sys

sys.path.insert(0, "docs/research/t4342")
import gen_probe  # noqa: E402

CASE = "eval/suites/probes/cases/t4342-groupby-tree-holdout"
PRED = "docs/research/t4342/predictions.tsv"


def tokens(text: str) -> list[list[object]]:
    """An ARRAYTOTEXT strict text `{a,b;c,d}` as rows of cells (quoted texts as str, the rest as numbers)."""
    body = text.strip()[1:-1]
    rows: list[list[object]] = [[]]
    i = 0
    while i < len(body):
        ch = body[i]
        if ch == '"':
            j, out = i + 1, []
            while j < len(body):
                if body[j] == '"' and j + 1 < len(body) and body[j + 1] == '"':
                    out.append('"')
                    j += 2
                elif body[j] == '"':
                    break
                else:
                    out.append(body[j])
                    j += 1
            rows[-1].append("".join(out))
            i = j + 1
        elif ch == ",":
            i += 1
        elif ch == ";":
            rows.append([])
            i += 1
        else:
            j = i
            while j < len(body) and body[j] not in ",;":
                j += 1
            raw = body[i:j].strip()
            rows[-1].append(int(raw) if raw.lstrip("-").isdigit() else raw)
            i = j
    return rows


def members(cell: object) -> list[int]:
    return [int(x) for x in str(cell).split(",") if x.strip()]


def canonical(column: str, text: str, nf: int) -> str:
    rows = tokens(text)
    if column == "E":
        cols = [str(c) for c in rows[0][nf:]]
        body = [[*map(str, r[:nf]), *[members(c) for c in r[nf:]]] for r in rows[1:]]
        return json.dumps({"columns": cols, "rows": body}, ensure_ascii=False)
    if column == "D":
        return json.dumps([[*map(str, r[:nf]), int(r[nf])] for r in rows], ensure_ascii=False)
    return json.dumps([[*map(str, r[:nf]), members(r[nf])] for r in rows], ensure_ascii=False)


def revised(label: str, column: str) -> str:
    """The revised model's canonical groups for the block a Calc label describes (`kind rows | p/q`)."""
    keys, pq = label.split("|")
    rows = [tuple(k) for k in keys.split()[1:]]
    nf = len(rows[0])
    if column == "E":
        return gen_probe.predict_pivotby(rows, list(pq.strip()))
    dirs = [-1] + [1] * (nf - 1) if column == "C" else [1] * nf
    return gen_probe.predict_groupby(rows, dirs, column == "D")


def main() -> None:
    with open(CASE + ".case.json") as fh:
        calc = {s["name"]: s["cells"] for s in json.load(fh)["sheets"]}["Calc"]
    with open(CASE + ".oracle.json") as fh:
        oracle = json.load(fh)["sheets"]["Calc"]
    if "--revised" in sys.argv:
        gen_probe.INCLUSIVE = True
    hits, misses = 0, []
    with open(PRED) as fh:
        for line in fh:
            if line.startswith("#"):
                continue
            cell, want = line.rstrip("\n").split("\t")
            if gen_probe.INCLUSIVE:
                want = revised(calc[f"A{cell[1:]}"], cell[0])
            nf = 3 if calc[f"A{cell[1:]}"].startswith("3f") else 2
            got = canonical(cell[0], oracle[cell]["v"], nf)
            if got == want:
                hits += 1
            else:
                misses.append((cell, calc[f"A{cell[1:]}"], want, got))
    print(hits, "/", hits + len(misses))
    for m in misses:
        print(*m, sep="\n    ")


if __name__ == "__main__":
    main()
