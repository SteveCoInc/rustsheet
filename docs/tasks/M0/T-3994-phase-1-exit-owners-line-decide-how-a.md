---
id: T-3994
title: "Phase 1 exit owners line: decide how a visibly-off cell with no math value, owned only by a deferred last-digit task, counts (61 cells today)"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A decision record says what a visibly-off probe cell needs to satisfy the `owners` line of `make phase1-exit` when two
things hold: R1(c) counts it only because it has no math value, and its only owner is a `defer: phase-1b` task. The
rule is applied to the cells of that kind today. Gap triage then stops re-deciding it case by case, sometimes filing
`visible` fit tasks for cells that a math value would classify as Phase 1b evidence.

## Context
- Gap triage 2026-11-03 (origin/main eda15c023, excel-graph). `scripts/phase1_exit.py` reports: "owners not met: 61
  visibly-off cells with no open owner (61 owned only by a deferred task); 0 milestone exit cells with no owner of
  their milestone". `make unowned-off ALL=1` lists them (60 after T-3993 was filed): CHIDIST 2, CHISQ.DIST.RT 2,
  POISSON 3 (T-3638's subnormal upper-tail probe, deferred owner T-3640), F.DIST.RT 7 and FDIST 1 (deferred owners
  T-2221, T-3199, T-3474, T-3527), GAMMALN 1 (deferred owners T-2122, T-3042, T-3331), and 45 unattributed
  `LN(GAMMA(x))` cells (deferred owners T-2122, T-3937).
- The rules do not agree:
  - Decision 0245's new-case gate passes a cell owned only by an open `defer:` task.
  - `phase1_exit.py`'s `owners` line, and `exit_owners.py --all-off`, count the same cell as unowned.
  - Decision 0114 and the gap-triage rule say `defer: phase-1b` is for cells already within 15 digits. A visibly-off
    cell gets a `campaign:` task.
  - Decision 0188 R1(c) leaves a visibly-off number out of the count as Phase 1b evidence once it has a math value and
    both Excel and the engine are within 1e-9 of it. A cell with no math value counts.
- These cells are R1(c)-counted only for want of a math value. They are one-call cells of functions with no Gate A
  definition yet (CHIDIST, POISSON, F.DIST.RT, FDIST, GAMMALN: T-3798 classifies them; CHISQ.DIST.RT is T-3925's
  table), or nested calls (T-3993). 50-digit mpmath at the doubles Excel reads puts every sampled one deep inside
  1e-9. For example:
  - `F.DIST.RT(1.61,1,50)`: Excel -4.7E-15 relative, ours +6.9E-17;
  - `CHIDIST(1479.2,21)`: Excel -5.0E-14, ours +1.5E-14;
  - `POISSON(19,777.9,TRUE)`: Excel -4.3E-14, ours +7.1E-14;
  - `LN(GAMMA(1.85))`: Excel -2.8E-15, ours -6.8E-16.
  So each would be Phase 1b evidence once it had a math value.
- Precedents that went the other way: T-3954 and T-3952 each took one gen-m9 cell out of deferred T-2221 into a
  `visible` task. gen-m9 is a Gate B suite, where every visibly-off cell counts whatever its math. That reason does
  not carry over to probe cells.
- Options to weigh, among others:
  - (a) Keep the line as is. Triage files a non-deferred owner (a campaign task) for each such group.
  - (b) A deferred owner suffices while an open, non-deferred task will attach the cell's math value. The cell's
    function is in the task's table list, or the cell's formula shape is in its scope.
  - (c) Leave such cells out of the owners line and show them in their own count, as R1(c) shows Phase 1b evidence.
  Any option that weakens a gate needs the decision record (CLAUDE.md, directive 2). Option (b) needs a mechanical
  test that `phase1_exit.py` can run.

## Acceptance criteria
- [x] A decision record (`make new-decision`) states the rule, with the counts above before and after applying it,
      and how `phase1_exit.py`, `exit_owners.py` and decision 0245's gate agree under it.
- [x] If the rule changes a script, the change is in this task's PR, with tests in scripts/test_phase1_exit.py or
      scripts/test_exit_owners.py. If it keeps the scripts and needs owners, the owner tasks are filed in the same PR.
- [x] .claude/agents/gap-triager.md and the steward skill (.claude/skills/steward/SKILL.md) state the rule where they
      describe `defer: phase-1b` (a protected-path change, if so, goes through the decision record above).
- [x] `make check` green; reviewers approve (reviewer-excel-fidelity included, since the rule touches R1(c)).

## Out of scope
- Writing math values (T-3993, T-3798's table tasks, T-3925) and any engine fit for these cells.
- Gate B's suites, where every visibly-off cell counts whatever its math (decision 0188 R1(b)).

## Outcome
- Decision 0325 (option (a), with the owner named): a visibly-off probes or probes-numeric cell that R1(c) counts only
  for want of a math value needs a non-deferred owner, and that owner is the task that attaches its math (its
  function's Gate A table task, or a math-sidecar task for its formula shape), never a `visible` or `accuracy` fit
  task before the math is attached. No gate or script changes; decision 0187 item 4 stands. Decision 0245's gate keeps
  passing a deferred-only cell at PR time (a lower bar than the exit's), and the record says how the three checks agree.
- The rule is in .claude/agents/gap-triager.md (next to `defer: phase-1b`) and the steward skill ("Deferred owners on
  the exit line").
- Counts: 61 such cells on 2026-11-03 (eda15c023); 0 today (origin/main e200fa21, `make eval` without calipers, poi or
  corpus-bench). The Gate A table tasks T-4103, T-4104, T-4106 and T-3925 and T-3993's composite sidecars attached
  their math, which is the rule's owner in practice; so no owner task was needed. `make phase1-exit` lists 2 cells
  owned only by a deferred task, both gen-m9 (Gate B, out of scope): `01523d247594952e!Calc!A377` (SYD over
  `F.INV.RT(1E-20,9,3)`) and `abc2939ebd1e2365!Calc!A11` (`CHIDIST(F.INV.RT(1E-200,1,1.999),10)`, Excel 0, ours
  `#NUM!`), from T-2808's split (PR #3715, still open); under item 4 they need a `campaign:` owner, which that PR's
  follow-ups or the next triage should give.
- Not script-changing, so no test in scripts/test_phase1_exit.py. Eval delta: none.
