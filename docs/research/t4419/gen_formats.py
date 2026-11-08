"""Build probes/t4419-band-formats: the 13 ties of probes/t4419-band-flip at n = -16..-10 (where every switch falls),
under five `.000` formats: s.000, ss.000, [s].000, mm:ss.000 and h:mm:ss.000. If the switch point moves with the
format, the decision depends on the field path, not only the value. python3 gen_formats.py CASE_PATH.
"""

import json
import sys

from_flip = [179.5, 203.5, 232.5, 253.5, 255.5, 258.5, 261.5, 266.5, 274.5, 291.5, 316.5, 320.5, 327.5]
FORMATS = ["s.000", "ss.000", "[s].000", "mm:ss.000", "h:mm:ss.000"]
cells = {}
row = 0
for t in from_flip:
    for fmt in FORMATS:
        row += 1
        for col, n in zip("ABCDEFG", range(-16, -9)):
            cells[f"{col}{row}"] = f'=TEXT({t}/86400000+({n})*2^(-71),"{fmt}")'
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4419: the band-flip ties under five .000 formats, n=-16..-10 (docs/research/t4419/gen_formats.py).",
    "tags": ["M10", "text", "probe"],
    "sheets": [{"name": "Sheet1", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as fh:
    fh.write(json.dumps(case, indent=2) + "\n")
print(len(cells), "cells")
