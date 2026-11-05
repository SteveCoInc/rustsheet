"""T-4029: T-3580's R4.1 record on the fit lab's own hash split (decision 0188 R4.1).

The cells are score_train.py's (cells.tsv's 3,435 F cdf/tail cells, c5.py's filter) together with every cell of the
hold-out probe probes-numeric t3567-r5-fdf1-hold whose Excel value is a nonzero number (3,532 of its 3,578), 6,967 in
all. Each goes to train or test by the fit lab's split (`rustsheet_tools.fitlab.holdout.is_holdout`: salt
rustsheet-fitlab-holdout/v1, 30%) on its case path (`probes-numeric/<name>.case.json`, as cells.tsv names it) and its
`Sheet!A1` address. Engine values and the correctly rounded value (mpmath, 50 digits) are score_train.py's. Prints
bit-exact / within 4 ulp / within 15 digits per side. Run with the tools venv (mpmath and the fit lab):
  cd tools && uv run python ../docs/research/t3580/split.py"""

import json
import math
from multiprocessing import Pool

from rustsheet_tools.fitlab.holdout import is_holdout
from rustsheet_tools.fitlab.sweep import Cell
from score_train import ROOT, correctly_rounded, engine_values, tally, train_rows

HOLD = "probes-numeric/t3567-r5-fdf1-hold.case.json"


def hold_rows():
    """(case, cell, x, m, n, upper, excel) of the hold-out probe's cells with a nonzero numeric Excel value."""
    cases = ROOT / "eval/suites/probes-numeric/cases"
    formulas = json.load(open(cases / "t3567-r5-fdf1-hold.case.json"))["sheets"][0]["cells"]
    oracle = json.load(open(cases / "t3567-r5-fdf1-hold.oracle.json"))["sheets"]["Calc"]
    rows = []
    for key, formula in formulas.items():
        value = oracle.get(key)
        if not value or value["t"] != "n" or value["v"] == 0:
            continue
        upper = not formula.startswith("=F.DIST(")
        args = formula[formula.index("(") + 1 : formula.rindex(")")].split(",")
        x, m, n = float(args[0]), math.trunc(float(args[1])), math.trunc(float(args[2]))
        rows.append((HOLD, f"Calc!{key}", x, m, n, upper, value["v"]))
    return rows


def held_out(case, cell):
    sheet, address = cell.split("!")
    return is_holdout(Cell(case=case, sheet=sheet, address=address, formula="", args=(), expected=math.nan))


def main():
    rows = train_rows() + hold_rows()
    actual = engine_values({row[0] for row in rows})
    with Pool() as pool:
        cr = pool.starmap(correctly_rounded, [row[2:6] for row in rows], chunksize=50)
    sides = {"train": ([], []), "test": ([], [])}
    for (case, cell, *_, e), c in zip(rows, cr):
        try:
            a = float(actual[(case, cell)])
        except (KeyError, ValueError):
            a = math.nan
        eng, ref = sides["test" if held_out(case, cell) else "train"]
        eng.append((a, e))
        ref.append((c, e))
    print(f"{len(rows)} cells")
    for side, (eng, ref) in sides.items():
        ex, w4, w15 = tally(eng)
        rex, rw4, rw15 = tally(ref)
        print(
            f"{side}: {len(eng)} cells; engine {ex}/{w4}/{w15} ({ex / len(eng):.1%}); correctly rounded {rex}/{rw4}/{rw15}"
        )


if __name__ == "__main__":
    main()
