"""Oracle probe t3582-r6-gamma-runs (probes-numeric): Excel's lower incomplete gamma P(b, z) = GAMMA.DIST(z, b, 1,
TRUE) along runs of 1,024 consecutive doubles in z, at the two shapes of the split probe's D runs (b = 0.7 and
b = 0.03, as the split probe states them) and at z0 = 0.3 and 1.0 (P well below 1, so its last-digit noise shows).
Each z is an exact double literal (w2-forensics' encoder). Column A = GAMMA.DIST. Meta: [[row, b, z, run], ...]
usage: mkprobe.py [--write]"""
from __future__ import annotations

import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r3-bgrat"))
from common import w2p, write_probe  # noqa: E402  (r3-bgrat's probe helpers, on the path above)

NAME = "t3582-r6-gamma-runs"
SHAPES = (0.7, 0.030000000000000027)
STARTS = (0.3, 1.0)
N = 1024

cells, meta = {}, []
for b in SHAPES:
    for z0 in STARTS:
        z = z0
        for _ in range(N):
            k = len(cells) + 1
            cells[f"A{k}"] = f"=GAMMA.DIST({w2p.num(z)},{w2p.num(b)},1,TRUE)"
            meta.append((k, b, z, f"b{b:g}-z{z0:g}"))
            z = math.nextafter(z, math.inf)
# write_probe stores the meta next to r3-bgrat/common.py; mkprobe moves it here.
write_probe(NAME, "T-3582 r6-bgrat-q: GAMMA.DIST(z, b, 1, TRUE) on runs of 1,024 consecutive doubles in z at b = 0.7 "
            "and 0.03, z0 = 0.3 and 1.0 (Excel's incomplete gamma noise at b < 1)", cells, meta, "--write" in sys.argv,
            ("T-3582", "incomplete-gamma"))
if "--write" in sys.argv:
    src = Path(__file__).resolve().parent.parent / "r3-bgrat" / f"{NAME}.meta.json"
    src.replace(Path(__file__).resolve().parent / src.name)
