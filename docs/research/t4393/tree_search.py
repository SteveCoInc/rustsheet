"""T-4393: score search-tree UNIQUE models on the 60 two-field runs of probes/t4379 and probes/t4380 (column B).

Run from the repo root: python3 -I docs/research/t4393/tree_search.py. A row is kept unless a lookup in the structure
of the rows kept so far finds it `=`: a sorted vector with lower- or upper-bound insertion (std::set style: the
bound's row is a duplicate when the key is not below it), and a plain or red-black BST with an equal-stop or a
lower-bound lookup. Result (2026-11-06): every model misses 34 or more of 60 (vec_lb 34, bst_lb 35, rb_lb 35,
rb_eq 36, bst_eq 37, vec_ub 42), against docs/research/t4393/model.py's merge, 0.
"""

import importlib.util
import sys

sys.path.insert(0, "docs/research/t4241")
import model

_spec = importlib.util.spec_from_file_location(
    "t4380_candidates", "docs/research/t4380/candidates.py"
)
assert _spec and _spec.loader
candidates = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(candidates)
rc = model.make_rcmp([1, 1])


def vec_lb(rows, upper=False, rev=False):
    a = []  # sorted list of indices
    idx = range(len(rows))
    if rev:
        idx = reversed(range(len(rows)))
    kept = []
    for i in idx:
        lo, hi = 0, len(a)
        while lo < hi:
            m = (lo + hi) // 2
            c = rc(rows[a[m]], rows[i])
            if (c < 0) if not upper else (c <= 0):
                lo = m + 1
            else:
                hi = m
        if not upper:
            if lo < len(a) and rc(rows[i], rows[a[lo]]) >= 0:
                continue
        else:
            if lo > 0 and rc(rows[a[lo - 1]], rows[i]) >= 0:
                continue
        a.insert(lo, i)
        kept.append(i)
    return sorted(kept)


def tree(rows, mode, balanced):
    """A BST of kept rows (children `ch[n]` = [left, right]); red-black (CLRS insertion) when `balanced`."""
    ch, up, red, root, kept = {}, {}, {}, [None], []

    def lookup(i):
        """(duplicate?, parent, side) for row i: an equal-stop walk, or a lower-bound walk that checks its bound."""
        n, parent, side, bound = root[0], None, 0, None
        while n is not None:
            parent = n
            if mode == "eqstop":
                c = rc(rows[i], rows[n])
                if c == 0:
                    return True, n, 0
                side = int(c > 0)
            else:
                side = int(rc(rows[n], rows[i]) < 0)
                if not side:
                    bound = n
            n = ch[n][side]
        return mode == "lb" and bound is not None and not rc(rows[i], rows[bound]) < 0, parent, side

    def rotate(x, d):
        """Rotate x down to side d (d = 0: left rotation)."""
        y = ch[x][1 - d]
        ch[x][1 - d] = ch[y][d]
        if ch[y][d] is not None:
            up[ch[y][d]] = x
        up[y] = up[x]
        if up[x] is None:
            root[0] = y
        else:
            ch[up[x]][ch[up[x]].index(x)] = y
        ch[y][d] = x
        up[x] = y

    for i in range(len(rows)):
        dup, parent, side = lookup(i)
        if dup:
            continue
        kept.append(i)
        ch[i], up[i], red[i] = [None, None], parent, True
        if parent is None:
            root[0] = i
        else:
            ch[parent][side] = i
        z = i
        while balanced and up[z] is not None and red[up[z]]:
            p = up[z]
            g = up[p]
            d = ch[g].index(p)
            uncle = ch[g][1 - d]
            if uncle is not None and red[uncle]:
                red[p] = red[uncle] = False
                red[g] = True
                z = g
                continue
            if z == ch[p][1 - d]:
                z = p
                rotate(z, d)
                p = up[z]
            red[p] = False
            red[g] = True
            rotate(g, 1 - d)
        red[root[0]] = False
    return sorted(kept)


MODELS = {
    "vec_lb": lambda r: vec_lb(r),
    "vec_ub": lambda r: vec_lb(r, True),
    "bst_eq": lambda r: tree(r, "eqstop", False),
    "bst_lb": lambda r: tree(r, "lb", False),
    "rb_eq": lambda r: tree(r, "eqstop", True),
    "rb_lb": lambda r: tree(r, "lb", True),
}

if __name__ == "__main__":
    cases = [
        (nm, i, [tuple(k) for k in run.split()], ex)
        for nm, i, run, ex in candidates.cases()
    ]
    for name, f in MODELS.items():
        miss = sum(
            " ".join("".join(rows[j]) for j in f(rows)) != ex
            for _, _, rows, ex in cases
        )
        print(name, miss)
