# 0334 — T-4434's huge-shape sweep is an exception to decision 0267 item 5's probe freeze

- **Status:** accepted (2026-11-07, T-4434, owner request).
- **Context:**
  - Decision 0267 item 5 allows a new probes-numeric case only for a task with a "Decision 0267" or "Decision 0267
    cells" section, and only for the cells it owns. Once those tasks close, probes-numeric freezes unless a decision
    names an exception.
  - T-4435 makes the incomplete beta accurate where one shape is huge but `large_shape_beta.rs` does not apply. That is
    region 1 (a0 = 5E3 to 1E6 at b0 < 40) and region 2 (b0 = 40 to 200 at a0 of 5E4 and up). The engine is 1E-13 to
    9E-8 off the math there. Decision 0114's `accuracy` stage, which T-4435 is, rests on Excel being near correctly
    rounded. The eval had almost no Excel cell in either region (T-4434's Context), so nobody could say per region
    whether Excel is the accurate side.
  - T-4434 is the probe that answers it, and T-4435 names it as its required evidence. Neither task has a "Decision 0267"
    section: they grew out of the 2026-11-06 gap triage, not decision 0267's tier lists.
  - The owner asked for both tasks, in order, in an interactive Claude Code session on 2026-11-07
    (https://claude.ai/code/session_01AeCw9bPvaKQB7VyiBFywWu): "T-4434 ... an oracle-filled probes-numeric sweep for
    F.DIST.RT, F.DIST and BETA.DIST, covering the two regions where the incomplete beta has a huge shape outside
    `large_shape_beta`", then T-4435.
- **Decision:** probes-numeric/t4434-huge-shape-outside-large-shape-beta (900 cells, written by
  docs/research/t4434/gen_probe.py, seed 4434) is an exception to decision 0267 item 5. The exception covers only that
  case and its oracle refills. It adds no cells to any other case.
- **Consequences:**
  - The case lands with its oracle and math sidecars. Its visibly-off cells go to an owner as T-4434's acceptance
    criteria say: T-4435 where Excel is near the math, T-4214 where Excel is the side off at b0 >= 40, T-3209 where it
    is off at b0 < 40. Its failing cells are new known failures of a new case (decision 0083), recorded through
    `make eval-baseline`.
  - When T-4057's freeze check lands, it must accept this case as a named exception, like decisions 0316 and 0320.
  - This sets no precedent for later sweeps. Another probes-numeric case outside decision 0267's tasks needs its own
    decision.
