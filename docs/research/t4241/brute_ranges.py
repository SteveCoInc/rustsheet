"""T-4241: no row order makes "group = S[lower_bound(k), upper_bound(k))" fit t4241's ascending blocks.

Run from the repo root: python3 docs/research/t4241/brute_ranges.py. For each block of
probes/t4241-groupby-intransitive-runs column B, it tries every order S of the rows. It asks whether the
whole-row binary-search ranges of the rows' keys contain each of Excel's member lists. It prints the
number of blocks some order fits (33 of 48).
"""
import itertools

from model import load, make_rcmp


def lb(s, k, rcmp):
    lo, hi = 0, len(s)
    while lo < hi:
        m = (lo + hi) // 2
        if rcmp(s[m][1], k) < 0:
            lo = m + 1
        else:
            hi = m
    return lo


def ub(s, k, rcmp):
    lo, hi = 0, len(s)
    while lo < hi:
        m = (lo + hi) // 2
        if rcmp(k, s[m][1]) < 0:
            hi = m
        else:
            lo = m + 1
    return lo


if __name__ == "__main__":
    rcmp = make_rcmp([1, 1])
    blocks = load("t4241-groupby-intransitive-runs", "B", 2)
    fit = 0
    for _, rows, want, _ in blocks:
        lists = [m for _, m in want]
        numbered = list(enumerate(rows, 1))
        for order in itertools.permutations(numbered):
            s = list(order)
            ranges = [sorted(i for i, _ in s[lb(s, k, rcmp):ub(s, k, rcmp)]) for _, k in numbered]
            if all(m in ranges for m in lists):
                fit += 1
                break
    print(fit, "/", len(blocks))
