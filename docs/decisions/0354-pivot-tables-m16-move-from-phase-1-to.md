# 0354 — Pivot tables (M16) move from Phase 1 to Phase 2

- **Status:** accepted (2026-11-08, owner ruling). Amends decision 0274 item 1. Items 2 to 5 stand: the lanes, the
  feature catalog as M16's scope list, the scope rulings and the oracle of record.
- **Context:**
  - Decision 0274 made M16 a Phase 1 milestone, and from then Phase 1 could not exit until M16's exit criteria
    held.
  - At main b84a7256 (2026-11-08), M16 is the largest open Phase 1 row by far:
    - gen-pivot has 3,077 visibly-off cells, pivot-corpus-lo 382 and pivot-corpus-closedxml 136;
    - its probe row has 26;
    - the evidence floor is met for 140 of 373 PV1 rows, 13 of 91 PV3 and 13 of 56 PV5;
    - the layout and corpus-render checks are not met.
  - The owner asked (2026-11-08) to move pivot tables back to Phase 2, so that Phase 1 is judged on the calculation
    engine's parity.
- **Decision:**
  1. M16 is a Phase 2 milestone. docs/milestones.md lists it with Phase 2, and `scripts/phase1_exit.py` no longer
     counts it among `PHASE1_MILESTONES`.
  2. The pivot work keeps everything it has: the M16 milestone tag on its tasks, the catalog and its owners, and the
     suites. The report's `milestone_exit` M16 row is still computed and shown every run (`milestone_exit.rs`
     `MILESTONES`), so the progress stays visible.
  3. Pivot features are no longer a Phase 1 exit criterion. The Phase 1 `owners` check still reads every
     visibly-off cell the Phase 1 rows count, so a pivot cell those rows count still needs an owner.
- **Consequences:**
  - `make phase1-exit` no longer prints a `milestone M16` line, and no longer fails on it.
  - M16 work runs alongside Phase 2 (M3, M4, M13, M15). Its tasks stay in the queue at their priorities, and
    nothing is dropped. Moving it back needs a new decision.
