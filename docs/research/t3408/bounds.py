"""T-3408: the 200-step IRR secant cap's bracket and range, from the train side of the fit lab split only.

long_runs.tsv lists every IRR literal cell with an oracle value (eval/suites and the Gate A IRR tables, 3,233 cells
on origin/main 73e09221b) whose secant run, replayed uncapped with the engine's own `irr_secant` arithmetic
(replay_test.rs), stops at its 51st to 300th step, in Excel's run order (the guess run, then the retry from 0.1).
Columns: the cell (case id as the fit lab keys it, Sheet!A1), its side (`fitlab.holdout.is_holdout`), the step the
run stops at, how it stops (`residual`: |step| and |NPV| under 1e-7; `stall`: a zero step at |NPV| above 1e-7), what
Excel shows (the run's value, the retry's value, another number, or its error), the flow count, the largest |flow|
and the guess.

Prints, from train cells only: the cap bracket (the last step whose residual stop Excel returns, the first step whose
run Excel does not finish), the range of the returning runs (flows, largest |c|, guess), the stall steps, and the same
counts on test as a check. Standard library only:

    python3 -I docs/research/t3408/bounds.py docs/research/t3408/long_runs.tsv
"""

from __future__ import annotations

import csv
import sys


def main() -> None:
    with open(sys.argv[1], newline="") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    for side in ("train", "test"):
        cells = [r for r in rows if r["side"] == side]
        returns = [
            r
            for r in cells
            if r["stop"] == "residual" and r["excel"] == "run's value" and int(r["step"]) <= 207
        ]
        unfinished = [r for r in cells if r["excel"] == "retry's value"]
        stalls = [r for r in cells if r["stop"] == "stall"]
        top = max(returns, key=lambda r: int(r["step"]))
        first = min(unfinished, key=lambda r: int(r["step"]))
        print(
            f"{side}: {len(returns)} residual stops at steps 51..{top['step']} return the run's value ({top['cell']})"
        )
        print(f"  first run Excel does not finish: step {first['step']} ({first['cell']})")
        for key, conv in (("flows", int), ("largest_c", float), ("guess", float)):
            lo = min(returns, key=lambda r: conv(r[key]))
            hi = max(returns, key=lambda r: conv(r[key]))
            print(f"  {key}: {lo[key]} ({lo['cell']}) .. {hi[key]} ({hi['cell']})")
        for r in sorted(stalls, key=lambda r: int(r["step"])):
            print(
                f"  stall at step {r['step']}: Excel {r['excel']}, {r['flows']} flows, |c| {r['largest_c']} ({r['cell']})"
            )


if __name__ == "__main__":
    main()
