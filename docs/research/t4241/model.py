"""T-4241: the GROUPBY grouping model that fits probes/t4195-groupby-intransitive and t4241-groupby-intransitive-runs.

Run from the repo root: python3 docs/research/t4241/model.py. A = ab, B = ab&U+00AD, C = U+00AD&ab:
A = C, B = C, A < B; x < y. The model:
1. UNIQUE's kept rows (T-4200's merge, `coalesce_equal_lines`), sorted by SORT's natural merge on whole rows
   (each field compared in its sort direction).
2. Classes are built field by field over that sorted list: under one parent, a key starts a new class unless its
   field `=` the field of the current class's first key. A class shows its first key's spelling.
3. Each data row descends the tree. At each field, a lower-bound binary search over the parent's class
   representatives (in the field's direction) finds a class. If the found representative is not `=` to the row's
   field, or the search runs past the end, the row is in no group.
4. A leaf class that no row reaches aggregates every row.
"""
import itertools
import json
import re

TEXT_NAMES = {'="ab"': "A", '="ab"&UNICHAR(173)': "B", '=UNICHAR(173)&"ab"': "C"}


def tcmp(a, b):
    if a == b:
        return 0
    s = {a, b}
    if s in ({"A", "C"}, {"B", "C"}):
        return 0
    if s == {"A", "B"}:
        return -1 if a == "A" else 1
    return -1 if a < b else 1


def make_rcmp(dirs):
    def rcmp(r, s):
        for f, (a, b) in enumerate(zip(r, s)):
            c = tcmp(a, b) * dirs[f]
            if c:
                return c
        return 0
    return rcmp


def natural_sort(items, rcmp):
    if not items:
        return []
    runs, cur = [], [items[0]]
    for it in items[1:]:
        if rcmp(it, cur[-1]) < 0:
            runs.append(cur)
            cur = [it]
        else:
            cur.append(it)
    runs.append(cur)
    while len(runs) > 1:
        nr = []
        for j in range(0, len(runs) - 1, 2):
            a, b, o = list(runs[j]), list(runs[j + 1]), []
            while a and b:
                o.append(b.pop(0) if rcmp(b[0], a[0]) < 0 else a.pop(0))
            nr.append(o + a + b)
        if len(runs) % 2:
            nr.append(runs[-1])
        runs = nr
    return runs[0]


def unique_kept(rows, rcmp):
    """UNIQUE's merge (T-4200): equal groups coalesce when they meet; returns kept row indices in input order."""
    groups = [[i] for i in range(len(rows))]
    runs, cur = [], [groups[0]]
    for g in groups[1:]:
        c = rcmp(rows[g[0]], rows[cur[-1][0]])
        if c == 0:
            cur[-1] = cur[-1] + g
        elif c < 0:
            runs.append(cur)
            cur = [g]
        else:
            cur.append(g)
    runs.append(cur)
    while len(runs) > 1:
        nr = []
        for j in range(0, len(runs) - 1, 2):
            a, b, o = list(runs[j]), list(runs[j + 1]), []

            def push(g):
                if o and rcmp(rows[o[-1][0]], rows[g[0]]) == 0:
                    o[-1] = o[-1] + g
                else:
                    o.append(g)
            while a and b:
                c = rcmp(rows[b[0][0]], rows[a[0][0]])
                if c == 0:
                    push(a.pop(0) + b.pop(0))
                elif c < 0:
                    push(b.pop(0))
                else:
                    push(a.pop(0))
            for g in a + b:
                push(g)
            nr.append(o)
        if len(runs) % 2:
            nr.append(runs[-1])
        runs = nr
    return sorted(min(g) for g in runs[0])


def groupby(rows, dirs, inclusive=False, ascending_search=False):
    """`inclusive`: T-4342's revision, the lower-bound search runs over [0, n-1] (it never probes past the last
    class); False is T-4241's frozen half-open search over [0, n)."""
    nf = len(rows[0])
    rcmp = make_rcmp(dirs)
    keys = natural_sort([rows[i] for i in unique_kept(rows, make_rcmp([1] * nf))], rcmp)

    def build(ks, f):
        if f == nf:
            return None
        classes = []
        for k in ks:
            if classes and tcmp(classes[-1][0][f], k[f]) == 0:
                classes[-1][1].append(k)
            else:
                classes.append((k, [k]))
        return [(c[0][f], build(c[1], f + 1)) for c in classes]

    tree = build(keys, 0)
    leaves = []

    def walk(t, path, shown):
        for i, (rep, sub) in enumerate(t):
            if sub is None:
                leaves.append((path + (i,), shown + (rep,)))
            else:
                walk(sub, path + (i,), shown + (rep,))
    walk(tree, (), ())
    members = {}
    for r, row in enumerate(rows, 1):
        t, path = tree, ()
        for f in range(nf):
            reps = [x[0] for x in t]
            lo, hi = 0, len(reps) - 1 if inclusive else len(reps)
            while lo < hi:
                m = (lo + hi) // 2
                if tcmp(reps[m], row[f]) * (1 if ascending_search else dirs[f]) < 0:
                    lo = m + 1
                else:
                    hi = m
            if lo == len(reps) or tcmp(reps[lo], row[f]) != 0:
                path = None
                break
            path, t = path + (lo,), t[lo][1]
        if path is not None:
            members.setdefault(path, []).append(r)
    return [(shown, members.get(p) or list(range(1, len(rows) + 1))) for p, shown in leaves]


def name_texts(s):
    for k, v in [("­ab", "C"), ("ab­", "B"), ('"ab"', '"A"')]:
        s = s.replace(k, v)
    return s.replace('"', "")


def parse(v, nf):
    gs = []
    for g in name_texts(v).strip("{}").split(";"):
        p = [x.strip() for x in g.split(",")]
        gs.append((tuple(p[:nf]), sorted(int(x) for x in p[nf:])))
    return gs


def load(case, col, nf):
    base = "eval/suites/probes/cases/" + case
    sheets = {s["name"]: s["cells"] for s in json.load(open(base + ".case.json"))["sheets"]}
    oracle = json.load(open(base + ".oracle.json"))["sheets"]["Calc"]
    out = []
    for r in range(1, 200):
        f = sheets["Calc"].get(f"{col}{r}")
        if not f or "GROUPBY" not in f:
            continue
        m = re.search(r"Data!A(\d+):[A-D](\d+)", f)
        a, b = int(m.group(1)), int(m.group(2))
        rows = []
        for i in range(a, b + 1):
            row = tuple(TEXT_NAMES.get(sheets["Data"].get(f"{c}{i}"), str(sheets["Data"].get(f"{c}{i}")).strip('="'))
                        for c in "ABC"[:nf])
            rows.append(row)
        if any(t not in ("A", "B", "C", "x", "y", "p", "q") for row in rows for t in row):
            continue
        out.append((f"{col}{r}", rows, parse(oracle[f"{col}{r}"]["v"], nf), f))
    return out


def sort_dirs(formula, nf):
    m = re.search(r",0,0,(-?\d)\)", formula)
    order = int(m.group(1)) if m else 1
    dirs = [1] * nf
    if order < 0:
        dirs[abs(order) - 1] = -1
    return dirs


if __name__ == "__main__":
    for case, col, nf in [("t4195-groupby-intransitive", "C", 2), ("t4195-groupby-intransitive", "E", 3),
                          ("t4241-groupby-intransitive-runs", "B", 2), ("t4241-groupby-intransitive-runs", "C", 2),
                          ("t4241-groupby-intransitive-runs", "D", 2), ("t4241-groupby-intransitive-runs", "E", 2)]:
        blocks = load(case, col, nf)
        bad = [(c, rows, want, groupby(rows, sort_dirs(f, nf))) for c, rows, want, f in blocks
               if groupby(rows, sort_dirs(f, nf)) != want]
        print(case, col, len(blocks) - len(bad), "/", len(blocks))
        for b in bad[:4]:
            print("   ", b)
