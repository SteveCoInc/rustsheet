---
id: T-4432
title: "Composite math follow-ups from T-3993's review: decision 0297's R3 caveat, score.py's output outside docs/, one paren scanner for composite.py and sweep.py"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The MINOR review findings of T-3993 (PR #3439) are resolved. Decision 0297 states that a composite math value is not
R3 (MATH-OVER-EXCEL) evidence on its own when its distance from Excel's value comes from inner rounding or flush, or
from Excel's snap of the outermost `+`/`-`. `docs/research/t3993/score.py` no longer leaves an untracked data file
in docs/. The quote, brace and parenthesis scanning rules each have one definition.

## Context
- The excel-fidelity review of #3439 (head 01d419102) flagged two kinds of composite cell where Excel's error against
  the math is above 1e-4 although Excel is not inaccurate:
  - t3566-beta-inv-tiny-shape-and-range Calc!A147 and eight more cells. Here the inner BETA.INV underflows and is
    flushed, so the math is 0 or 1.
  - About 921 cells whose outermost `+`/`-` Excel snaps to 0 (numbers.md "Near-zero snapping"). One is
    t3677-sumx-running-overflow-snap Calc!A16.

  Nothing says these cannot back a MATH-OVER-EXCEL ruling (decision 0188 R3). An amendment to the accepted decision
  goes through `make new-decision` if the decision process requires a new record. Check docs/decisions/README.md
  first.
- `score.py` writes `cells.tsv` next to itself, under docs/research/t3993/, which is not git-ignored. Write it under
  eval/runs/ or a temporary directory instead.
- The design review found that `composite._closing` (tools/src/rustsheet_tools/accuracy/composite.py) repeats the
  quote and bracket scanning of `fitlab/sweep.py`'s `split_args`. One shared scanner in sweep.py should serve both.
  Then run `scripts/check_duplication.py --update`, which went to 2876 against a baseline of 2878 on #3439.

## Acceptance criteria
- [x] Decision 0297 (or the record the decision process requires) states the R3 caveat, citing the two kinds of
      cell above.
- [x] Running `docs/research/t3993/score.py` leaves no untracked file under docs/.
- [x] `composite._closing` and `sweep.split_args` share one scanner. The existing tests pass, and
      `make eval-math-check` stays current with no sidecar byte changed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to which cells get a composite math value, or to the values themselves.

## Outcome
- Decision 0297 is amended in place, under Consequences: "Not R3 evidence on its own". Decision records are amended
  in place (docs/decisions/README.md needs a new record only for protected paths and baseline allowances; 0188 is
  amended the same way). The amendment says a composite cell more than 1e-4 from its (b) value backs no
  MATH-OVER-EXCEL entry when the distance comes from rounding the math does not model. It cites both kinds of cell:
  - an inner call flushed to 0 or 1 (t3566-beta-inv-tiny-shape-and-range Calc!A147 and eight more cells);
  - Excel's near-zero snap of the outermost `+`/`-` (about 921 cells, among them t3677-sumx-running-overflow-snap
    Calc!A16).
- `docs/research/t3993/score.py` writes `cells.tsv` to the git-ignored `eval/runs/t3993/`. After a run,
  `git status docs/` is clean.
- `fitlab.sweep.unquoted` is now the one quote and bracket scanner. `split_args` and `composite._closing` both use
  it, and a unit test pins it. `make eval-math-check` reports both sidecars current, with no sidecar byte changed.
  Duplication went from 2876 to 2872 (`quality/duplication.json` updated with `--update`).
- Eval delta: none (tools and docs only).
- Follow-up filed with this PR: T-4706, the wording MINORs of the excel-fidelity review of #3697.
