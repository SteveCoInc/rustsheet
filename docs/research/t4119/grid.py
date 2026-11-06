"""T-4119's grid: 60-digit F tails at a huge df2 (written as grid-references.json).

Run with tools' venv: `tools/.venv/bin/python docs/research/t4119/grid.py > grid-references.json`.
"""

import json

import mpmath as mp

mp.mp.dps = 60
pts = []
for d1 in (3, 5, 8, 12, 20, 40, 60, 79):
    for d2 in (2.1e6, 5e6, 1e7, 2e7, 5e7):
        for x in (0.5, 0.9, 1.0, 1.1, 1.5, 2.0, 3.0, 5.0):
            D1, D2, X = mp.mpf(d1), mp.mpf(d2), mp.mpf(x)
            up = mp.betainc(D2 / 2, D1 / 2, 0, D2 / (D2 + D1 * X), regularized=True)
            lo = mp.betainc(D1 / 2, D2 / 2, 0, D1 * X / (D2 + D1 * X), regularized=True)
            for upper, v in ((True, up), (False, lo)):
                if v > mp.mpf("1e-120"):
                    pts.append((upper, x, d1, d2, mp.nstr(v, 30)))
print(json.dumps(pts))
