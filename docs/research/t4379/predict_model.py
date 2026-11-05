"""T-4379: the GROUPBY tree model's predictions (docs/research/t4241/model.py, inclusive search) for
probes/t4379-pivotby-repeated-unreached-leaf, written before the fill to docs/research/t4379/model_predictions.tsv.

Run from the repo root: python3 -I docs/research/t4379/predict_model.py. Per run: the keys UNIQUE keeps (column B, as
A/B/C and x/y in source order) and GROUPBY's groups with their members (column C). The PIVOTBY columns' predictions
are the engine's (predict_engine.py), which builds its column tree with the same model.
"""

import importlib.util
import sys

sys.path.insert(0, "docs/research/t4241")
import model  # noqa: E402

_spec = importlib.util.spec_from_file_location("t4379_gen_probe", "docs/research/t4379/gen_probe.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

OUT = "docs/research/t4379/model_predictions.tsv"


def main() -> None:
    lines = ["row\trun\tunique_kept\tgroupby"]
    for r, (run, _) in enumerate(gen.RUNS, start=1):
        rows = [tuple(k) for k in run.split()]
        kept = model.unique_kept(rows, model.make_rcmp([1, 1]))
        groups = model.groupby(rows, [1, 1], inclusive=True)
        unique = " ".join("".join(rows[i]) for i in kept)
        grouped = "; ".join("".join(k) + ": " + ", ".join(map(str, ms)) for k, ms in groups)
        lines.append(f"{r}\t{run}\t{unique}\t{grouped}")
    with open(OUT, "w") as fh:
        fh.write("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
