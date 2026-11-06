---
id: T-4456
title: "BETA.DIST takes T-4045's L140 prefactor zero rule (T-4437's first part)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The worksheet BETA.DIST cdf is Excel's exact 0 wherever T-4045's rule applies: L140 bup+bgrat with bup's prefactor ret
below DBL_MIN. F.INV's search keeps today's kernel.

## Context
- Split from T-4437. T-4437 keeps the earlier cut at few bup terms and the t's cut.
- Evidence: probes/t4254-beta-t-l140-zero-cut A12:A22 and C12:C22 (T-4254, PR #3469) are 0 from ln ret -709. A
  parked branch from T-4254 (agent/functions-stat/T-4254-beta-t-zero-edge-probe, PR #3464 closed as a duplicate) has
  two more filled probes: n = 25 and 29 bup terms at a = 5E6, and the earlier cut against b, a and df. It also has the
  engine change: `bratio::l140_prefactor_zero_tails` asked from `beta.rs`, so `beta_dist_lower_tail` stays unchanged
  for F.INV's search (accuracy/F.INV/s1 A146).

## Acceptance criteria
- [x] BETA.DIST's cdf is 0 at the rule's cells of probes/t4254-beta-t-l140-zero-cut, and at those of the salvaged
      probe; unit test; accuracy/F.INV/s1 A146 is unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Engine: the worksheet BETA.DIST cdf (`beta.rs`) now asks `bratio::l140_prefactor_zero_tails`, T-4045's rule, which is
  now `pub(crate)`. Where L140 bup+bgrat's prefactor ret is below DBL_MIN, the cdf is Excel's exact 0. Excel's
  complement is 1, after a swap. `bratio::beta_dist_lower_tail` is unchanged, so F.INV's search
  (`beta_lower_crossing`) and accuracy/F.INV/s1 A146 do not move. Unit test: `l140_prefactor_zero_is_beta_dists_zero`.
- Cells fixed:
  - probes/t4254-beta-t-l140-zero-cut A12:A22, B12:B22, C12:C22 and D12:D22 (from #3469). The n = 3 families B and D are
    past ln ret -709 from row 12 on, so the rule covers them there too.
  - probes/t4456-beta-t-zero-cut C10:C17 and D10:D17 (n = 25 and 29 bup terms, past the n <= 9 port).
- Probes, salvaged from the parked T-4254 branch (PR #3464, closed as a duplicate of #3469) and renamed:
  probes/t4456-beta-t-zero-cut (130 cells) and probes/t4456-beta-t-zero-reach (81 cells), excel-graph, 2026-11-06.
  statistics.md "Where the exact 0 starts (T-4045)" gains a "More shapes" bullet. It finds:
  - the earlier cut also reaches n = 5, and lies in w between 2.8E-295 and 8.5E-291 at every shape probed;
  - the t's cut moves with df;
  - Excel's F.INV root at A146 lies in its own cdf's flat 0.
- Owners: T-4437 (scope narrowed to the earlier cut and the t) owns the visibly-off cells of the salvaged probes;
  T-4442 (filed) owns cut G1 (F.INV). `scripts/new_case_owners.py` passes.
- Eval delta: 44 known failures fixed (t4254-beta-t-l140-zero-cut), 0 regressions. The 2 new cases add 211 cells: 91
  pass and 120 are known failures, owned by T-4437 (the earlier zeros), T-4442 (F.INV, cut G1) and T-4463 (30 cells
  of 13th-digit tail residue). `scripts/new_case_owners.py` passes.
- Follow-ups: T-4463; T-4442; T-4457 (sonnet: T-4223's statistics.md wording, the MINORs of PR #3475's fidelity review).

## Landing notes
- PR #3480, head c7c8a3931. Every reviewer approved that head: security, design and excel-fidelity.
- CI is red on one job, eval (Excel-compatibility gate), in its `make eval-math-check` step:
  https://github.com/SteveCoInc/rustsheet/actions/runs/42162454930/job/127711255693. The two new cases,
  probes/t4456-beta-t-zero-cut and t4456-beta-t-zero-reach, have no `*.math.json` sidecars ("has no math value and is
  not listed as skipped"). Every other job is green. `make check-fast` does not run this gate.
- Next step: `cd tools && uv run rustsheet-accuracy math --suite probes`. The run needs more than 5 minutes; my
  background run was stopped by its time limit before it reached the t4456 cases. Commit only the two
  `t4456-beta-t-zero-*.math.json` files.
  - The full run also rewrites m9-bessel-order-limits, t2546-bessel-past-cap and t2881-besselj-turning-point
    sidecars, which `--check` does not report as stale. Leave those out.
  - Then run `cd tools && uv run rustsheet-accuracy math --check --suite probes --suite probes-numeric`, push, and
    wait for CI. A sidecar is reference data, not behaviour: re-run excel-fidelity only if the math values contradict
    statistics.md's "the math there is 1.05E-264" (C10). Then enqueue.
- Still owed after merge: a task-only PR adding #3480's review MINORs to T-4457 (listed in the PR comment of
  2026-11-06): the salvaged cases' T-4254 description and tags, the decision 0188 R3 mention in T-4463, and the
  mpmath source of 1.05E-264.
