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


def bst(rows, mode):
    # unbalanced BST
    L = {}
    R = {}
    root = None
    kept = []
    for i in range(len(rows)):
        if root is None:
            root = i
            kept.append(i)
            L[i] = R[i] = None
            continue
        n = root
        bound = None
        parent = None
        dup = False
        goleft = False
        while n is not None:
            c = rc(rows[i], rows[n])
            if mode == "eqstop":
                if c == 0:
                    dup = True
                    break
                parent = n
                goleft = c < 0
                n = L[n] if c < 0 else R[n]
            else:  # lower_bound: if node<key right else left
                parent = n
                if rc(rows[n], rows[i]) < 0:
                    goleft = False
                    n = R[n]
                else:
                    bound = n
                    goleft = True
                    n = L[n]
        if mode == "lb" and bound is not None and not rc(rows[i], rows[bound]) < 0:
            dup = True
        if dup:
            continue
        kept.append(i)
        L[i] = R[i] = None
        if goleft:
            L[parent] = i
        else:
            R[parent] = i
    return sorted(kept)


# Red-black tree (CLRS insertion) with lower_bound or eqstop search
def rbt(rows, mode):
    L = {}
    R = {}
    P = {}
    col = {}
    root = [None]
    kept = []

    def rotl(x):
        y = R[x]
        R[x] = L[y]
        if L[y] is not None:
            P[L[y]] = x
        P[y] = P[x]
        if P[x] is None:
            root[0] = y
        elif x == L[P[x]]:
            L[P[x]] = y
        else:
            R[P[x]] = y
        L[y] = x
        P[x] = y

    def rotr(x):
        y = L[x]
        L[x] = R[y]
        if R[y] is not None:
            P[R[y]] = x
        P[y] = P[x]
        if P[x] is None:
            root[0] = y
        elif x == R[P[x]]:
            R[P[x]] = y
        else:
            L[P[x]] = y
        R[y] = x
        P[x] = y

    for i in range(len(rows)):
        n = root[0]
        bound = None
        parent = None
        dup = False
        goleft = False
        while n is not None:
            parent = n
            if mode == "eqstop":
                c = rc(rows[i], rows[n])
                if c == 0:
                    dup = True
                    break
                goleft = c < 0
                n = L[n] if c < 0 else R[n]
            elif mode == "lb":
                if rc(rows[n], rows[i]) < 0:
                    goleft = False
                    n = R[n]
                else:
                    bound = n
                    goleft = True
                    n = L[n]
        if mode == "lb" and bound is not None and not rc(rows[i], rows[bound]) < 0:
            dup = True
        if dup:
            continue
        kept.append(i)
        L[i] = R[i] = None
        P[i] = parent
        col[i] = "r"
        if parent is None:
            root[0] = i
        elif goleft:
            L[parent] = i
        else:
            R[parent] = i
        z = i
        while P[z] is not None and col[P[z]] == "r":
            p = P[z]
            g = P[p]
            if p == L[g]:
                y = R[g]
                if y is not None and col[y] == "r":
                    col[p] = col[y] = "b"
                    col[g] = "r"
                    z = g
                else:
                    if z == R[p]:
                        z = p
                        rotl(z)
                        p = P[z]
                    col[p] = "b"
                    col[g] = "r"
                    rotr(g)
            else:
                y = L[g]
                if y is not None and col[y] == "r":
                    col[p] = col[y] = "b"
                    col[g] = "r"
                    z = g
                else:
                    if z == L[p]:
                        z = p
                        rotr(z)
                        p = P[z]
                    col[p] = "b"
                    col[g] = "r"
                    rotl(g)
        col[root[0]] = "b"
    return sorted(kept)


MODELS = {
    "vec_lb": lambda r: vec_lb(r),
    "vec_ub": lambda r: vec_lb(r, True),
    "bst_eq": lambda r: bst(r, "eqstop"),
    "bst_lb": lambda r: bst(r, "lb"),
    "rb_eq": lambda r: rbt(r, "eqstop"),
    "rb_lb": lambda r: rbt(r, "lb"),
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
