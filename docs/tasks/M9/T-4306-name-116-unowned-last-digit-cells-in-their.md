---
id: T-4306
title: "Name 114 unowned last-digit cells in their Phase 1b kernel tasks: t3192-beta-half-half-2 in T-4071, F.TEST in T-2765"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
Each cell below is named, beside its case, in the open Phase 1b task that owns its kernel, and that task names T-4306.
After that, the 2026-11-05 triage's ownership check finds no failing cell of these cases without an owner.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 642d7f82). Every failing cell was checked against the open tasks
  with `scripts/exit_owners.py`'s ownership rule (`owned`, `report_case_names`). These cells are within 15 digits of
  Excel (1 to 4 ulp), so `make unowned-off` does not list them, and no open task names them by cell. Each belongs to a
  kernel that already has an open `defer: phase-1b` task. The work is bookkeeping in docs/tasks/ only: no engine
  change and no new evidence. That is why it is `sonnet`, and `defer: phase-1b` like its targets.
- **BETA.DIST at a = b = 1/2 (113) → T-4071.** T-4071 scores the three t3192 sweeps by name ("-2") but lists no cells.
  Against the case's `.math.json`, Excel is 0.54 ulp from the math (median) and ours 0.58. Example B102
  `BETA.DIST(A102,0.5,0.5,TRUE)`: Excel 0.7535270457844029, ours ...031.
  - probes-numeric/t3192-beta-half-half-2.case.json (113): Calc!B14, B20, B32, B37, B39, B43, B54, B63, B65, B72, B84, B88, B89, B92, B93, B94, B96, B102, B104, B105, B107, B109, B112, B116, B117, B118, B123, B135, B137, B139, B141, B144, B145, B147, B148, B149, B151, B155, B157, B158, B159, B160, B161, B165, B167, B172, B175, B176, B182, B183, B186, B189, B192, B193, B194, B196, B198, B199, B202, B205, B207, B208, B210, B213, B214, B215, B217, B218, B219, B221, B223, B225, B228, B230, B232, B233, B234, B239, B240, B241, B244, B247, B250, B251, B253, B258, B260, B266, B267, B269, B278, B281, B284, B286, B288, B292, B296, B299, B302, B303, B304, B307, B308, B310, B312, B314, B315, B316, B325, B327, B330, B334, B339
- The TREND and GROWTH cells of t3185-b A62 and t3185-c A69 are T-4281's (open), not this task's.
- **F.TEST (1) → T-2765.** `F.TEST(Data!A7:D8,Data!C8:E10)` is Excel 1.8768433447187109E-93 and ours ...106E-93.
  - gen-m9/8dd2312c96effb1b.case.json (1): Calc!A341
- Use the cell-list line format of docs/process/task-format.md. Add a "Split (T-3229 check)" note naming T-4306 in each
  target, so `tasks.py validate` accepts the shared listing.
- Left out on purpose:
  - The two MIRR cells of the power root probe are the web side of a documented desktop-versus-web conflict
    (financial.md, MIRR "Web vs desktop"). Decision 0266 item 2 says a triage files no task for it.
  - One IRR cell of T-3411's second R4 fresh sweep (row 138; Excel 0.3838361826002541, ours ...543) has no open Phase 1b
    IRR digit task to go to. T-3408's target is 15 digits, and this cell already meets it. The triage report records
    it, and this task does not own it.

## Acceptance criteria
- [ ] T-4071 and T-2765 each list the cells above that are routed to them, beside their case, and each names
      T-4306.
- [ ] Running the ownership check as the triage did (`owned` over `open_task_texts`, after `make eval`) leaves none of
      these 114 cells unowned once T-4306 is `done`.
- [ ] `python3 scripts/tasks.py validate` passes; only files under docs/tasks/ change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any kernel or baseline. The MIRR and IRR cells named above.
