"""T-4244: predictions for probes/t4244-fv-pv-nonint-nper, written before the excel-graph fill.

Columns: the engine's value (from `rustsheet-eval show` on the case before the fill, passed as a file), and the three
candidates of docs/research/t4146: twice-rounded operators (H_twice, the engine's form), plain doubles (H_plain) and
the correctly rounded value (H_exact, 80-digit mpmath rounded once).
Run (from tools/): uv run python ../docs/research/t4244/predict.py <engine-show.txt> > ../docs/research/t4244/predictions.tsv
"""

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "t4146"))
import forms  # noqa: E402
import score  # noqa: E402

CASE = HERE.parents[2] / "eval/suites/probes/cases/t4244-fv-pv-nonint-nper.case.json"


def main() -> None:
    shown = Path(sys.argv[1]).read_text()
    engine = dict(re.findall(r"Calc!(A\d+)\s+\w+\n\s+formula : .*\n\s+expected: .*\n\s+actual  : (.*)", shown))
    cells = json.loads(CASE.read_text())["sheets"][0]["cells"]
    print("cell\tformula\tengine\tH_twice\tH_plain\tH_exact")
    for cell in sorted(cells, key=lambda c: int(c[1:])):
        f = cells[cell]
        m = re.fullmatch(r"=(FV|PV)\((.*)\)", f)
        assert m
        func, args = m[1], [float(a) for a in m[2].split(",")]
        ev = forms.fv if func == "FV" else forms.pv
        row = [repr(ev("twice", *args)), repr(ev("plain", *args)), repr(score.exact(func, *args))]
        print("\t".join([cell, f, engine.get(cell, "?"), *row]))


main()
