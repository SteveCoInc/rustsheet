# 0338 — T-4857's F family equivalence-class probe is an exception to decision 0267 item 5's probe freeze

- **Status:** accepted (2026-11-07, T-4857; the owner's request of 2026-11-07).
- **Context:**
  - Decision 0267 item 5 allows a new probes-numeric case only for a task with a "Decision 0267" or "Decision 0267
    cells" section, and only for the cells it owns. T-4857 has no such section and owns no Tier A or B cells.
  - The owner asked for this probe on 2026-11-07, verbatim: "do some thorough analysis of the arguments and their
    types and their ranges, and thinking about the general shape of the functions involved here, and then come up
    with a probe that tries really hard to do a nice job of getting a full set of equivalence classes across all of
    the different kind of interesting ranges of the input arguments, so that we can have more confidence that we're
    capturing all of the interesting points to spot areas of discontinuities and nonlinearities that might otherwise
    easily escape our extrapolation confidence."
  - The concern is overfitting: the F family's Excel evidence is clustered where earlier research tasks probed. The
    coverage audit (docs/research/t4857/COVERAGE.md, `audit.py`) found 218 of the 441 (df1, df2) class pairs of the
    F tails, 210 of the density's and 326 of the quantiles' with no Excel cell before this task.
  - The probe's numeric half has to live in probes-numeric: its cells are one-call numeric cells whose math sidecars
    feed Gate A's envelope reporting, as every other F sweep's do. Its argument-kind half goes to `probes`, which
    the freeze does not cover.
- **Decision:** probes-numeric/t4857-f-classes (3,281 cells, generator docs/research/t4857/gen.py, seed 4857) is an
  exception to decision 0267 item 5's probe freeze, by the owner's request of 2026-11-07. Later oracle refills of its
  cells are allowed as for any case; new cells in it need their own decision.
- **Consequences:**
  - T-4057's freeze check, once it lands, lists this case as a named exception.
  - Its cells that the engine gets wrong enter eval/baseline through `make eval-baseline`, and each has an owner task
    (decision 0245). Family cells first wrong at the 14th or 15th digit are Phase 1b residue under decision 0267
    item 2; they get no new work.
  - No other probes-numeric case is opened by this decision.
