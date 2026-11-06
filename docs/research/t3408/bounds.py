"""T-3408: how the 200-step IRR secant cap and its range are chosen, from the fit lab's train side only.

Three files, all written by replay_test.rs (an uncapped and a per-cap replay of the engine's own `irr_secant` and
`compute_irr`) over every IRR literal cell with an oracle value (eval/suites and the Gate A IRR tables, 3,233 cells,
origin/main 73e09221b). Each row carries its fit lab side (`fitlab.holdout.is_holdout`, keys `suite/case` and
`Sheet!A1`, Gate A cells `accuracy/IRR/<batch>`).

- cap_runs.tsv: the cells whose result changes with the cap applied inside `IRR_LONG_RUN_RANGE`, with the engine's
  result at each candidate cap (`cap_<n>`), the uncapped guess run and retry from 0.1, and Excel's value.
- long_runs.tsv: every cell whose first finishing secant run (the guess run, else the retry) stops at its 51st..300th
  step, uncapped and without the range: the step, how it stops (`residual` or `stall`), what Excel shows (the run's
  value, the retry's value, another number, or its error) and whether the run and the retry give the same bits
  (`yes`: the cell cannot tell them apart).
- stalls.tsv: every stalled stop (a zero step at |NPV| above 1e-7, T-3120's stop), at any step.

Excel's outcome is not a clean function of the step count: on train, runs of 241..291 steps return their own value
while runs of 217..287 steps do not. So no cap is Excel's; 200 is a point on the train-score plateau. This prints:
the train and test score of each candidate cap (inside the shipped range); the train cells past 200 steps by outcome;
the range of the returning train runs up to the chosen cap, first over the cells where run and retry give different
bits (the shipped range, rounded outward: largest |c| up to 2.2E+8, guess from -0.99934), then over all of them; and
the stall evidence.

    python3 -I docs/research/t3408/bounds.py docs/research/t3408
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

CHOSEN = 200


def read(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def same(excel: str, value: str) -> bool:
    try:
        return float(excel) == float(value)
    except ValueError:
        return excel == value


def caps(rows: list[dict[str, str]]) -> None:
    names = [k for k in rows[0] if k.startswith("cap_")]
    print("cap   train matched   test matched   (cells whose result depends on the cap)")
    for name in names:
        train = sum(same(r["excel"], r[name]) for r in rows if r["side"] == "train")
        test = sum(same(r["excel"], r[name]) for r in rows if r["side"] == "test")
        print(f"{name[4:]:>4}  {train:5}           {test:5}")
    n_train = sum(r["side"] == "train" for r in rows)
    print(f"      of {n_train} train and {len(rows) - n_train} test cells")


def past_cap(rows: list[dict[str, str]]) -> None:
    print(f"train runs past {CHOSEN} steps (residual stops), by what Excel shows:")
    for r in sorted(rows, key=lambda r: int(r["step"])):
        if r["side"] == "train" and r["stop"] == "residual" and int(r["step"]) > CHOSEN:
            print(
                f"  step {r['step']:>3}  {r['excel']:<14} run = retry: {r['run_equals_retry']:<8} {r['cell']}"
            )


def bounds(rows: list[dict[str, str]], label: str) -> None:
    print(label, f"({len(rows)} cells)")
    for key, conv in (("flows", int), ("largest_c", float), ("guess", float)):
        lo = min(rows, key=lambda r: conv(r[key]))
        hi = max(rows, key=lambda r: conv(r[key]))
        print(f"  {key}: {lo[key]} ({lo['cell']}) .. {hi[key]} ({hi['cell']})")


def main() -> None:
    folder = Path(sys.argv[1])
    caps(read(folder / "cap_runs.tsv"))
    long_runs = read(folder / "long_runs.tsv")
    past_cap(long_runs)
    returning = [
        r
        for r in long_runs
        if r["side"] == "train"
        and r["stop"] == "residual"
        and r["excel"] == "run's value"
        and int(r["step"]) <= CHOSEN
    ]
    bounds(
        [r for r in returning if r["run_equals_retry"] != "yes"],
        f"range of the train runs of 51..{CHOSEN} steps that return their own value, run and retry differing (shipped)",
    )
    bounds(returning, "  the same, with the cells where run and retry give the same bits (not used)")
    print("stalled stops past step 30 (all sides):")
    for r in sorted(read(folder / "stalls.tsv"), key=lambda r: int(r["step"])):
        if int(r["step"]) > 30:
            print(
                f"  step {r['step']:>3}  {r['side']:<5} {r['excel']:<12} {r['flows']} flows, |c| {r['largest_c']}  {r['cell']}"
            )


if __name__ == "__main__":
    main()
