"""T-4359: GROUPBY's sort by a value on intransitive runs: the hypotheses for a parent's value.

The tree, the search and the leaves are docs/research/t4344/gen_probe.py's (`groupby` with H3, the model the engine
implements); this adds each row's descent path, the classes it reached from the first field (fewer than the fields
when it reached no leaf). A value sort builds its tree ascending and orders hierarchically: the parents (key
prefixes) by their value, the groups by their own value inside each parent, ties in key (tree) order. The
hypotheses differ only in the rows a parent's value is the function of:
- R0: the rows of its groups (a leaf no row reaches holds every row): the engine before T-4359.
- R1: the rows whose descent reached its class, those that reached no leaf under it included, and not the extra
  rows of a leaf no row reaches (t4344 Calc!D56; PIVOTBY's column subtotals, T-4378).
- R2: R0 and R1 together (the rows of its groups and the rows that reached it).
- R3: R1, except that a class no row reached holds every row, as a leaf no row reaches does (added after the fill
  of probes/t4359-groupby-value-sort-parents, from its subtotals, Calc!E4, E10: a repeated first-field class no row
  reached shows the total of every row).
Texts: A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B).
"""

import importlib.util
from collections.abc import Callable

_spec = importlib.util.spec_from_file_location("t4344_gen_probe", "docs/research/t4344/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

tcmp = gen.tcmp


def tree_of(rows: list[tuple[str, ...]]):
    """The ascending tree (a value sort's), its leaves (path, shown keys) in tree order, and each row's path."""
    nf = len(rows[0])
    m = gen.model
    keys = m.natural_sort([rows[i] for i in m.unique_kept(rows, m.make_rcmp([1] * nf))], m.make_rcmp([1] * nf))

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
    paths = []
    for row in rows:
        t, path = tree, ()
        for f in range(nf):
            reps = [x[0] for x in t]
            i = gen.h3(reps, row[f], 1)
            if tcmp(reps[i], row[f]) != 0:
                break
            path, t = path + (i,), t[i][1]
        paths.append(path)
    return leaves, paths


def value_sort(
    rows: list[tuple[str, ...]], agg: Callable[[list[int]], float], ascending: bool, hyp: str
) -> tuple[list[tuple[tuple[str, ...], float]], dict[tuple[int, ...], float]]:
    """GROUPBY's value-sorted groups (shown keys, value) under hypothesis `hyp`, and each parent's value. `agg` gets
    1-based row positions in source order."""
    leaves, paths = tree_of(rows)
    nf = len(rows[0])
    n = len(rows)
    every = list(range(1, n + 1))
    members = {p: [r for r in every if paths[r - 1] == p] for p, _ in leaves}
    held = {p: (ms or every) for p, ms in members.items()}
    parents: dict[tuple[int, ...], float] = {}
    for p, _ in leaves:
        for level in range(1, nf):
            prefix = p[:level]
            if prefix in parents:
                continue
            r0 = sorted({r for q, ms in held.items() if q[:level] == prefix for r in ms})
            r1 = [r for r in every if paths[r - 1][:level] == prefix]
            rs = {"R0": r0, "R1": r1, "R2": sorted(set(r0) | set(r1)), "R3": r1 or every}[hyp]
            parents[prefix] = agg(rs)
    sign = 1 if ascending else -1

    def key(item):
        p, _ = item
        return tuple(sign * parents[p[:level]] for level in range(1, nf)) + (sign * agg(held[p]),)

    order = sorted(leaves, key=key)  # stable: ties stay in tree (key) order
    return [(shown, agg(held[p])) for p, shown in order], parents
