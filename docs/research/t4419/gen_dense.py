"""Build probes/t4419-band-dense: every millisecond tie t = k + 0.5 ms for k = 165..328 (all of the 2^-19 band) at
t/86400000 + n*2^-71 days, n = -16..-10, under "s.000". One row per tie, columns A:G for n = -16..-10."""
import json, sys
cells = {}
for row, k in enumerate(range(165, 329), 1):
    for col, n in zip("ABCDEFG", range(-16, -9)):
        cells[f"{col}{row}"] = f'=TEXT({k}.5/86400000+({n})*2^(-71),"s.000")'
case = {"format": "rustsheet-case/v1",
        "description": "T-4419: every millisecond tie of the 2^-19-day band at n=-16..-10 ulps under s.000 (gen_dense.py).",
        "tags": ["M10", "text", "probe"], "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}}
open(sys.argv[1], "w").write(json.dumps(case, indent=2) + "\n")
print(len(cells), "cells")
