# 0266 — Desktop-oracle work is out of scope; Phase 4 tasks are dropped

- **Status:** accepted (2026-11-04, owner request)
- **Context:**
  - The owner (2026-11-04), on the 47 blocked tasks: "Desktop work is out of scope, please close."
  - Decision 0039 parked work that needs desktop Excel in Phase 4 (M14) as `status: blocked`
    until `rustsheet-oracle status` shows `excel-desktop OK`. The backend was never set up, and
    25 of the 47 blocked tasks on `main` waited on it, which made the blocked count a poor
    signal of real stalls.
- **Decision:**
  1. Work that needs evidence only desktop Excel can give is out of scope. Every M14 task that is
     not done is set to `status: dropped`, and so is any other task whose only blocker is the
     `excel-desktop` backend. Phase 4 and M14 are closed; no new task is filed there.
  2. excel-graph (Excel for the web) is the only oracle (decision 0039 rule 2 stands without its
     Phase 4 qualifier). A behavior excel-graph cannot observe stays as implemented, with its
     UNVERIFIED or CONFLICTING note in docs/excel-semantics/; a gap triage does not file a task for
     it.
  3. Decision 0039 rules 2–4 stay in force for Phases 1–3. Its rule 1 (Phase 4) is superseded. A
     reference to T-1074, T-1075 or another dropped M14 task in a task or semantics entry is a
     record of an open question, not a dependency.
  4. If the owner later sets up the desktop backend, a new decision reopens the scope; the dropped
     task files keep their context for that.
- **Consequences:**
  - The dropped tasks: T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174, T-1204,
    T-1277, T-1299, T-1310, T-1367, T-1553, T-1928, T-2182, T-2325, T-2425, T-2663, T-2704,
    T-2716, T-2856, T-3314, T-3750, T-3865, T-3899, T-3901. The blocked count falls by 25.
  - T-3300 stays `todo`: it reads an Excel-authored workbook already in upstream IronCalc and
    needs no desktop backend.
  - docs/milestones.md marks Phase 4 / M14 as out of scope.
