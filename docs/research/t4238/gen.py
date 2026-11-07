"""T-4238 probe: NEGBINOM.DIST's cdf on bfrac at a huge number_s, where Excel is #NUM! at 1E+20 and 0 at 1E+15.

Run: python3 -I docs/research/t4238/gen.py <case.json> <predictions.tsv>
Row r: number_s s from 1E+15 to 1E+20. Columns per failures f in (40, 100, 1000), p = 0.323212154487823:
  NEGBINOM.DIST(f, s, p, TRUE), BETA.DIST(p, s, f+1, TRUE), BETA.DIST(p, s, f+1, FALSE).
H_absorb: the cdf is #NUM! exactly where fl(s + (f + 1)) == s (f + 1 is lost in the sum of the shapes).
H_dens: the cdf is #NUM! exactly where Excel's density at the same point is #NUM! (read off the fill).
"""

import json
import sys

S = ["1E+15", "1E+16", "1E+17", "3E+17", "5E+17", "6E+17", "8E+17", "1E+18", "2E+18", "5E+18", "9E+18", "1E+19",
     "2E+19", "1E+20"]
F = [40, 100, 1000]
P = "0.323212154487823"
cols = "ABCDEFGHI"
cells: dict[str, str] = {}
out = ["row\tnumber_s\tfailures\tH_absorb"]
for i, s in enumerate(S, 1):
    for k, f in enumerate(F):
        cells[f"{cols[3 * k]}{i}"] = f"=NEGBINOM.DIST({f},{s},{P},TRUE)"
        cells[f"{cols[3 * k + 1]}{i}"] = f"=BETA.DIST({P},{s},{f + 1},TRUE)"
        cells[f"{cols[3 * k + 2]}{i}"] = f"=BETA.DIST({P},{s},{f + 1},FALSE)"
        a = float(s)
        out.append(f"{i}\t{s}\t{f}\t{'#NUM!' if a + (f + 1) == a else '0'}")
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "NEGBINOM.DIST's cdf on bfrac at a huge number_s (T-4238), predictions in docs/research/t4238 committed before "
        "the fill. Row r: number_s 1E+15 to 1E+20. Columns A-C, D-F, G-I: failures 40, 100, 1000, each with "
        "NEGBINOM.DIST(f,s,p,TRUE), BETA.DIST(p,s,f+1,TRUE) and BETA.DIST(p,s,f+1,FALSE), p 0.323212154487823. "
        "H_absorb: #NUM! exactly where fl(s + f + 1) = s; H_dens: exactly where the density is #NUM!."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": "Calc", "cells": cells}],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
