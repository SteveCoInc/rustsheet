"""Generate probes/t4241-groupby-intransitive-runs (T-4241).

Run from the repo root: python3 docs/research/t4241/gen_probe.py. Texts: A = ab, B = ab&U+00AD,
C = U+00AD&ab (A = C, B = C, A < B). Each Calc row is one block of Data rows: A the text, B x/y,
D the row's position in the block, E 2^(position-1). Calc B: GROUPBY of A:B, ascending, ARRAYTOTEXT
aggregate (lists the member positions); C: descending (sort_order -1); D: sorted by the second field
(sort_order 2); E: by the second field descending (-2); F: SUM of E, ascending (a row in two groups
is summed in each); G: UNIQUE of A:B.
"""
import itertools
import json
import random

TEXTS = {"A": '="ab"', "B": '="ab"&UNICHAR(173)', "C": '=UNICHAR(173)&"ab"'}
OUT = "eval/suites/probes/cases/t4241-groupby-intransitive-runs.case.json"


def blocks():
    out = []
    # The 18 three-row blocks of T-4195 (every order of A, B, C; x at one row, y elsewhere), fresh.
    for perm in itertools.permutations("ABC"):
        for xat in range(3):
            out.append(("3r " + "".join(perm) + f" x@{xat + 1}", [(t, "x" if i == xat else "y") for i, t in enumerate(perm)]))
    # Seeded four-row runs over A, B, C (each text present), second field x/y.
    rng = random.Random(4241)
    seen = set()
    while len(out) < 18 + 30:
        texts = [rng.choice("ABC") for _ in range(4)]
        if set(texts) != set("ABC"):
            continue
        fields = [rng.choice("xy") for _ in range(4)]
        if len(set(fields)) < 2:
            continue
        key = (tuple(texts), tuple(fields))
        if key in seen:
            continue
        seen.add(key)
        out.append(("4r " + "".join(texts) + " " + "".join(fields), list(zip(texts, fields))))
    return out


def main():
    data, calc = {}, {}
    row = 1
    for r, (label, rows) in enumerate(blocks(), start=1):
        a, b = row, row + len(rows) - 1
        for i, (t, f) in enumerate(rows):
            n = row + i
            data[f"A{n}"] = TEXTS[t]
            data[f"B{n}"] = f'="{f}"'
            data[f"D{n}"] = i + 1
            data[f"E{n}"] = 2**i
        keys, pos, pw = f"Data!A{a}:B{b}", f"Data!D{a}:D{b}", f"Data!E{a}:E{b}"
        calc[f"A{r}"] = label
        for col, order in (("B", 1), ("C", -1), ("D", 2), ("E", -2)):
            calc[f"{col}{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pos},LAMBDA(v,ARRAYTOTEXT(v)),0,0,{order}),1)"
        calc[f"F{r}"] = f"=ARRAYTOTEXT(_xlfn.GROUPBY({keys},{pw},LAMBDA(v,SUM(v)),0,0),1)"
        calc[f"G{r}"] = f"=ARRAYTOTEXT(_xlfn.UNIQUE({keys}),1)"
        row = b + 1
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4241: GROUPBY on multi-field intransitive runs (A = ab, B = ab&U+00AD, C = U+00AD&ab; "
        "A = C = B, A < B). Rows 1-18: the 18 three-row blocks of probes/t4195-groupby-intransitive, fresh. "
        "Rows 19-48: seeded four-row runs. B: ascending, ARRAYTOTEXT of the member positions; C: sort_order -1; "
        "D: sort_order 2; E: sort_order -2; F: SUM of 2^(position-1); G: UNIQUE.",
        "tags": ["M7", "probe", "text"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": calc}, {"name": "Data", "cells": data}],
    }
    with open(OUT, "w") as fh:
        json.dump(case, fh, indent=1, ensure_ascii=False)
        fh.write("\n")


if __name__ == "__main__":
    main()
