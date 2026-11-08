---
id: T-5096
title: "make catch-up resolves eval/baseline/usage-gates.txt conflicts (T-5093 follow-up)"
milestone: M0
lane: harness
status: todo
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
- [ ] `resolvable_lists` includes eval/baseline/usage-gates.txt, and a conflict there is resolved to main's side
  before the re-record.
- [ ] A test in scripts/ shows a conflicted tier lock is resolved, and a branch that turns a tier unmet still
  fails the catch-up's gate.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tier computation and the lock format (T-5093, T-5092).
