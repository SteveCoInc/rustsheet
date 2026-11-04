# 0269 — The engine lane takes up to three tasks in flight at once until Phase 1 exits

- **Status:** accepted (2026-11-04, owner request, T-4082). It amends 0022 (one task in flight per
  lane) for the `engine` lane only.
- **Context:**
  - The owner's 2026-11-04 push to exit Phase 1 raised the owners of the failing milestone exit
    cells to P0 and made `engine` the only critical lane (#3126). Ten of the sixteen P0 owners are
    `engine` tasks: M2 (T-4040, T-4024), M5 (T-3891, T-3997, T-3950), M7 (T-3418) and M8 (T-4020,
    T-1683, T-3856).
  - `scripts/tasks.py` `available()` treats a lane as busy while any of its tasks has a branch that
    moved in the last 2 h, whatever the worker's tier. So the scheduled workers (sonnet A, sonnet B,
    opus) and any adopting session could run only one engine task at a time, and the exit blockers
    ran in series while the workers took other lanes.
  - 0022's reason was to keep parallel workers off each other's files. The functions work avoids
    that by splitting into nine `functions-*` lanes, but the engine tasks above touch different parts
    (collation, numeral digits, tables, R1C1 names), and the merge queue (0247) catches a conflict
    before it lands.
- **Decision:** `LANE_SLOTS = {"engine": 3}` in `scripts/tasks.py`: a lane is busy once it has as
  many in-flight tasks (branches that moved in the last 2 h, not done on `main`, not unresumed
  handoffs) as it has slots. Every other lane keeps one slot. Claims are unchanged: a task claimed by
  a pushed branch is never offered to another worker.
- **Consequences:**
  - Up to three engine tasks run at once, enough for every scheduled worker to take one.
  - Two engine PRs may conflict more often; the queue drops the later one, and its session or an
    adopter (0248, 0253) merges `main` in and enqueues it again.
  - Revisit when Phase 1 exits: lower the slots back to one, or split `engine` into sub-lanes.
