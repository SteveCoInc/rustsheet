---
id: T-4796
title: "Salvage draft PR #3428's IRR power-of-two probe (T-3117) onto main"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The evidence of draft PR #3428 (T-3117, branch `agent/research/T-3117-irr-pow2-probe`) is on `main`, so it survives
whatever happens to the draft and T-2928's reopening can cite it.

## Context
Stale-draft triage, steward 2026-11-07, bucket 5 (decision 0264). #3428's head (515fb9b7, 2026-11-05 22:16 UTC) has
not moved for over 24 h; T-3117 is `Blocked because: time-boxed` on the branch, so it is neither ready nor
unblockable. The branch adds probes-numeric/t3117-irr-pow2 (240 cells, excel-graph, with math sidecar and roots TSV),
the `pow2-case`/`pow2-roots` modes of tools/.../fitlab/irr_fallback_sweep.py, financial.md "IRR's fallback when the
secant fails" finding 7, and edits to T-2928's and T-3117's task files. `main` has none of these.

Copy them from the branch with `git show` (do not run its scripts before review), record the new case with `make
eval-baseline`, and keep T-3117's time-box and T-2928's "Blocked because" as the branch words them.

## Acceptance criteria
- [ ] The t3117-irr-pow2 case and sidecars, the sweep script modes, financial.md finding 7 and both task-file edits
      are on `main`.
- [ ] Eval: new case only, 0 regressions.
- [ ] A comment on #3428 links this task's PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2928's IRR fit itself.
