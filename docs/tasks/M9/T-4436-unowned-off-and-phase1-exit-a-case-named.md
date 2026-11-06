---
id: T-4436
title: "unowned-off and phase1-exit: a case named in passing by a live task hides the visibly-off cells only a deferred or dropped task lists (34 t4045 F.DIST.RT cells, GAMMALN A3188, INFO B3)"
milestone: M9
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`make unowned-off ALL=1` and `make phase1-exit`'s owners criterion (decision 0187) stop counting a passing mention of
a case as ownership of its visibly-off cells. A cell that a deferred or dropped task lists by its A1 name, and that a
live task only names as part of a whole case, is reported as needing a live owner, so gap triage sees it. The rule is
recorded in a decision record that amends decision 0159.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 39f04d556, every suite fetched). `make unowned-off ALL=1` printed
  "0 visibly-off cells have no open owner" and phase1-exit's owners row was met. A per-cell look at who owns each
  visibly-off cell (`scripts/exit_owners.py` `owned` per task) found these held only by an incidental whole-case
  mention:
  - probes/t4045-f-dist-rt-zero-cut-fresh (24 F.DIST.RT cells): A1, A5, A6, B1, B6:B8, C1, C2, C6, C7, D1, D6, E1:E4,
    E7, E8, F1, F4, F6, F8, F9.
    - They are listed by T-3209 (deferred).
    - The live mention is T-4280, a sonnet tidy-up of review notes that cites the case as evidence for a comment.
  - probes-numeric/t4045-f-dist-rt-zero-cut (10 F.DIST.RT and FDIST cells): A3, A15, B8, B9, B15, B18, C1, C2, C3, E3.
    - E3 is listed by T-3209 (deferred).
    - The live mention is T-4254, a BETA.DIST and T.DIST.RT probe that cites the case as evidence.
  - probes-numeric/t2464-r2-bpser-small-gl Calc!A3188 (GAMMALN).
    - It is listed by T-2122 (deferred).
    - The live mention is T-3764's copied "T-3992 (excel-graph fill-to-fill stability ...) measured on
      t2464-r2-bpser-small-gl" bullet. That bullet sits in 59 open tasks, each naming its case as a whole.
  - calipers/roundtrip/formula_stress_test Information!B3 (`INFO(C3)`, desktop "16.0", web `#VALUE!`).
    - Its owner T-2663 was dropped (decision 0266).
    - The live mention is T-2906, whose own Context says the INFO cells are not its own.
  - T-4207 already records one more case of the kind: gen-m9/589f42e67f4b4b8d A329, "owned today only through an
    incidental mention of its case in T-3485".
  - The same gap triage gave each of these cells a real owner: T-4076 (the 33 F.DIST.RT cells and A3188), T-4214 (E3)
    and T-4207 (Information!B3). The tool gap remains.
- Why it happens: `names_whole_case` owns every cell of a case on any line that names the case and has no A1 token
  (decisions 0096, 0136, 0159). Evidence bullets, review notes and copied context lines name cases that way. Of the
  1,175 visibly-off cells that no live task lists by A1 name, most are owned through a live task's whole-case line.
  Most of those are the case's real owner (T-3412 and T-3403 on t3311, T-3469 on t3284), so "never count whole-case
  names" is not a rule either.
- A rule to confirm or replace with reasons:
  - A whole-case name does not own a cell that some other open or dropped task lists by A1 name beside the case,
    unless the naming task also lists cells of that case.
  - Report such a cell in `--all-off` as `<function> (listed only by deferred T-xxxx)` or
    `(listed only by dropped T-xxxx)`, and count it in phase1-exit's owners row.
  - Alternatives to compare: a marker for evidence-only mentions (for example a `(evidence)` suffix the parser skips);
    or whole-case ownership only from list-item lines in a cells section.
- Code: `owned`, `names_whole_case`, `unowned_off`, `main_all_off` in scripts/exit_owners.py; scripts/phase1_exit.py
  (owners row); scripts/desktop_owned.py uses the same `owned` (decision 0159 "Consequences"). Tests:
  scripts/test_exit_owners.py. `scripts/` is a protected path, hence the decision record.
- Related, not overlapping: T-3438 (a case that fails whole, owned by the block that lists its cells), T-3822 and
  T-3823 (partial runs, `--function` narrowing). If T-3438 lands first, rebase onto its block rule.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) amends decision 0159 with the rule chosen. Decision 0159 is not edited,
      and the new record names it in its Status line. The record compares at least two options and says how
      desktop_owned.py's list is affected.
- [ ] scripts/test_exit_owners.py has tests of each shape above:
  - an evidence line naming a case whose cells only a deferred task lists: the cells are reported, and flagged as
    deferred-listed;
  - the same with a dropped task;
  - a live task naming the case as a whole and also listing some of its cells: the cells are owned, unchanged;
  - a live task that is the case's only namer and lists no cell: the cells are owned, unchanged;
  - decision 0159's leak test keeps passing.
- [ ] On a fresh full `make eval`, the PR body lists every cell whose result changes between owned and unowned, with
      its new owner. Each newly unowned cell gets an owner in the same PR, or a follow-up task is filed for it.
      `make unowned-off ALL=1` and `make phase1-exit`'s owners row agree.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rewriting the 59 copied T-3992 bullets, or other tasks' evidence lines, beyond what a newly unowned cell needs.
- Changing which cells count as visibly off (decision 0188 R1(c)).
