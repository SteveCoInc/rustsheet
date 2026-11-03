"""T-3582 r6-bgrat-q qscan: does a different Q(b, z) algorithm at the start of Excel's direct bgrat explain D's white
noise? r3-bgrat's x87 bgrat (bgld.bgrat_ld, stored lnx and z) with j0 from each candidate of qalg.py, in doubles,
taken either as the fraction itself (hj) or through a rounded Q = fl(r_d * h) divided by the extended r (qj).
Prints exact/w4/w15 on the split probe's 2,908 points (tune and hold halves) and the sd of Excel - model on its two
1,024-double runs, against r3-bgrat's exact-Q and grat1-CF baselines."""
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
sys.path.insert(0, str(HERE))
import bgld  # noqa: E402  (r3-bgrat's x87 bgrat, on the path above)
import bscore  # noqa: E402
import qalg  # noqa: E402
from common import split_cells, table  # noqa: E402

RUNS = ((30.7, 0.7), (40.0, 0.030000000000000027))
EPS = 1e-15  # bgrat's series stop, r3-bgrat's best


def hj(fn, eps):
    return lambda b, z, _r: fn(b, z, eps)


def qj(fn, eps):
    return lambda b, z, r: float(np.float64(qalg.r_of(b, z)) * np.float64(fn(b, z, eps))) / r


def candidates():
    yield "exact", "exact"
    yield "grat1 cf ld", "cf"
    for e in (2.220446049250313e-16, 1e-15, 2.220446049250313e-15, 3e-7):
        yield f"nr_gcf {e:g} hj", hj(qalg.nr_gcf, e)
        yield f"nr_gcf {e:g} qj", qj(qalg.nr_gcf, e)
    for e in (1.1102230246251565e-16, 2.220446049250313e-16):
        yield f"cephes {e:g} hj", hj(qalg.cephes_igamc, e)
        yield f"cephes {e:g} qj", qj(qalg.cephes_igamc, e)


def main():
    C = {(c["a"], c["b"], c["x"]): c for c in bscore.cells()}
    rows = split_cells()
    g = defaultdict(list)
    sd = defaultdict(list)
    for name, qm in candidates():
        for a, b0, x, _case, d in rows:
            c = C[(a, b0, x)]
            m = bgld.bgrat_ld(a, b0, c["lnx"], qm, EPS)
            half = "hold" if c["row"]["hold"] else "tune"
            g[(name, half)].append((m, d))
            g[(name, "ALL")].append((m, d))
            if (a, b0) in RUNS:
                sd[(name, (a, b0))].append((d - m) / d * 2.0 ** 53)
    table(g, ("Q", "half"))
    print()
    for k, v in sorted(sd.items(), key=str):
        v = np.array(v)
        print(k, f"mean {v.mean():+.2f} sd {v.std():.2f}")


if __name__ == "__main__":
    main()
