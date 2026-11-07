import json, math, random, sys
rng = random.Random(3740_2026_11_07)
rows = []
def draw(zlo, zhi):
    while True:
        n = int(math.exp(rng.uniform(math.log(1e4), math.log(2**31 - 2))))
        p = round(rng.uniform(0.02, 0.98), 12)
        mu, sd = n * p, math.sqrt(n * p * (1 - p))
        z = rng.uniform(zlo, zhi)
        s1 = math.floor(mu + z * sd) + 1
        s2 = s1 + int(rng.uniform(0, 3) * sd)
        if 1 <= s1 <= s2 <= n:
            return n, p, s1, s2
for _ in range(40): rows.append(draw(0.1, 3.0))
for _ in range(20): rows.append(draw(3.0, 8.0))
for _ in range(10): rows.append(draw(-1.0, -0.05))
cells = {}
for i, (n, p, s1, s2) in enumerate(rows, 1):
    cells[f"A{i}"] = f"=BINOM.DIST.RANGE({n},{p!r},{s1},{s2})"
    cells[f"B{i}"] = f"=BINOM.DIST({s2},{n},{p!r},TRUE)-BINOM.DIST({s1 - 1},{n},{p!r},TRUE)"
case = {
    "format": "rustsheet-case/v1",
    "description": ("BINOM.DIST.RANGE past the mean at a large n (T-3740), a fresh sweep drawn after the "
                    "upper-tail form froze (seed 374020261107). n log-uniform in [1E4, 2^31-2], p in [0.02, 0.98]; "
                    "rows 1-40 start at mean + 0.1..3 sd (the band t3444 never probed), 41-60 at 3..8 sd, "
                    "61-70 just below the mean (controls). A BINOM.DIST.RANGE(n,p,s1,s2), "
                    "B Excel's own cdf difference BINOM.DIST(s2)-BINOM.DIST(s1-1)."),
    "tags": ["M9", "statistical", "binomial", "probe"],
    "sheets": [{"name": "Range", "cells": cells}],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
