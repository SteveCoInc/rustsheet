---
id: T-5096
title: "make catch-up resolves eval/baseline/usage-gates.txt conflicts (T-5093 follow-up)"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: [T-5093]
---

## Goal
`make catch-up` (and `make renumber`) resolves a conflict in eval/baseline/usage-gates.txt like the other
eval-gated lists, instead of refusing the merge.

## Context
- T-5093 added eval/baseline/usage-gates.txt, the usage-tier lock (decision 0355 item 6): one
  `foundational|expert<TAB>met|unmet` line per tier, written only by `rustsheet-eval update-baseline`
  (`gate_tiers::Lock`).
- scripts/merge_catchup.py `resolvable_lists` / `checked_baselines` list the cell baselines and the
  visible-parity lock, not the tier lock, so a branch and main that both changed it conflict outside the
  resolvable set and the catch-up stops. Rare: the file changes only when a tier flips between met and unmet.
- The right resolution is the same as for the cell baselines: take main's side, then let the re-recording
  `update-baseline` write the branch's state, which refuses met → unmet without `--allow-regressions`.

## Acceptance criteria
- [x] `resolvable_lists` includes eval/baseline/usage-gates.txt, and a conflict there is resolved to main's side
  before the re-record.
- [x] A test in scripts/ shows a conflicted tier lock is resolved, and a branch that turns a tier unmet still
  fails the catch-up's gate.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tier computation and the lock format (T-5093, T-5092).

## Outcome
- `scripts/merge_catchup.py`:
  - `USAGE_GATES` (eval/baseline/usage-gates.txt) is in `checked_baselines`, so `resolvable_lists` (shared by
    `make catch-up` and `make renumber`) resolves a conflict there to `main`'s side before the re-record, like the
    cell baselines;
  - after the re-record, `tier_flips` refuses a merged lock that has a tier unmet (or missing) that `main` has met.
    `update-baseline` already refuses met → unmet without `--allow-regressions`, which a catch-up never passes;
    this holds the pushed tree to it whatever the re-record did (decision 0355 item 6).
- Tests (scripts/test_merge_catchup.py): a conflicted lock takes `main`'s side and is re-recorded; a branch that
  leaves a tier unmet where `main` has it met is refused and nothing is pushed; `tier_flips` on its own. 34
  catch-up and renumber tests pass.
- docs/process/quality-gates.md: the usage-tier paragraph says how a catch-up treats the lock.
- Eval delta: none.
