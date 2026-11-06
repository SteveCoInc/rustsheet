---
id: T-4486
title: "make unowned-off FAILING=1: every failing cell no open task names, last-digit cells included, counted per case"
milestone: M9
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Gap triage can list, with one command after `make eval`, every failing cell of report.json that no open task owns.
That includes the last-digit cells that are not visibly off, which `make unowned-off ALL=1` cannot list. Ownership
uses the `owned` rule `--all-off` already uses (decisions 0136, 0138, 0139, 0159).

## Context
- `scripts/exit_owners.py --all-off` lists only visibly-off cells. Most Phase 1b owner tasks were found with ad-hoc
  scripts over report.json's `cases[].cells`, run by hand each time:
  - T-4365's Context: "An ownership check over every failing cell of report.json";
  - T-4336, T-4337, T-4377, T-4405, T-4406, T-4407 and T-4412: "cells ... no task names";
  - gap triage 2026-11-06, which filed T-4484 for 2,327 t2513 tdensity cells.
- In the 2026-11-06 run (origin/main d0ba57191, every suite fetched) the hand scan took 70 s.
  - It walked every failing cell (verdict not in `PASSING`) and kept those `owned` by no open task, deferred tasks
    included.
  - It found 3,598 items. 1,085 of them were pending cases with no cell (`calipers`, `corpus-bench`, `poi`), which are
    noise here.
  - The rest were 2,327 t2513 tdensity cells, 130 CUMPRINC underflow cells already listed on the pushed T-3141
    branch, 42 t4045 family-residue cells and 14 poi import-error cases.
- Code: `scripts/exit_owners.py`. Reuse `failing_cells`, `report_case_names`, `owned`, `open_task_texts` and
  `unowned_off`'s per-case narrowing of task texts. Do not write a second matcher (CLAUDE.md directive 3).
  `make unowned-off` is in the Makefile (line 180 at d0ba57191).
- T-4436 (todo, opus) changes how `owned` treats a whole-case mention. This task only calls `owned`, so the two do
  not conflict. If T-4436 lands first, the new listing inherits its rule.

## Acceptance criteria
- [ ] `scripts/exit_owners.py --all-failing` (and `make unowned-off FAILING=1`) prints one line per unowned failing
      cell, as `case!Sheet!A1<TAB>verdict<TAB>formula<TAB>expected<TAB>actual`, with `(deferred)` after the verdict
      when only a deferred task owns it, as `--all-off` does.
  - Then one count line per case, with its verdict counts.
  - Then a total line, `N failing cells have no open owner`.
- [ ] Cells whose verdict is in `PASSING` are left out. So are cases whose status is `pending` and that have no
      failing cell. A case that fails whole (engine error, invalid case, panic, timeout) is one `*` line, as
      `ExitCell`'s `WHOLE_CASE` is elsewhere.
- [ ] `--strict` exits 1 when the total is not 0. Without it the exit code is 0, as for `--all-off`.
- [ ] Unit tests in `scripts/test_exit_owners.py`, each on a small synthetic report and tasks directory:
  - a last-digit `numeric_ulp` cell no task names is listed;
  - one a live task names is not listed;
  - one only a deferred task names is listed with `(deferred)`;
  - a pending case with no cell is not listed.
- [ ] The Makefile help line for `unowned-off` names `FAILING=1`, and `scripts/test_makefile_titles.py` passes.
- [ ] On a full `make eval` at the PR's head, the command finishes in under 120 s. The PR body gives its total and
      top five cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule itself (T-4436, decision 0159).
- Making `phase1-exit` or any gate read the new listing. It is a triage aid and gates nothing.
