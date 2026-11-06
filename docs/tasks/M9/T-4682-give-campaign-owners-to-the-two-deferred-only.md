---
id: T-4682
title: "Give campaign owners to the two deferred-only gen-m9 cells on the phase1-exit owners line (SYD over F.INV.RT, CHIDIST of F.INV.RT)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The two gen-m9 cells that `make phase1-exit`'s owners line lists as owned only by a deferred task have an open
`campaign:` owner (decision 0325 item 4, decision 0188 R1(b)), so the line lists none.

## Context
- T-3994 (2026-11-06, origin/main e200fa21, `make eval` without calipers, poi or corpus-bench): "owners not met:
  2 visibly-off cells with no open owner (2 owned only by a deferred task)". `scripts/exit_owners.py --all-off`:
  - gen-m9/01523d247594952e Calc!A377 `SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`:
    Excel -1987062826671.328, ours -1987062826671.3352 (the F.INV.RT far tail, visibly off at 15 digits);
  - gen-m9/abc2939ebd1e2365 Calc!A11 `CHIDIST(F.INV.RT(1E-200,1,1.999),10)`: Excel 0, ours `#NUM!` (an error where
    Excel gives a number: a `semantics` group).
- Both came out of T-2808's split (PR #3715; its deferred follow-ups such as T-4662 own them). Gen-m9 is a Gate B
  suite, so a `defer: phase-1b` owner does not take them off the owners line (decision 0325 item 4).

## Acceptance criteria
- [ ] Each cell is named (decision 0136) in an open non-deferred task with a `campaign:` stage: `semantics` for A11,
      `visible` or `accuracy` (with mpmath evidence) for A377, filed or amended.
- [ ] `python3 scripts/exit_owners.py --all-off` lists neither after `make eval`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the cells.
