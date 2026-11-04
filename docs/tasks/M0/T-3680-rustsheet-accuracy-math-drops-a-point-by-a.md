---
id: T-3680
title: "rustsheet-accuracy math drops a point by a 10 s wall clock, so sidecars depend on the machine: use a deterministic budget"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math` writes the same sidecars on any machine for the same inputs, as `write_suite` claims.

## Context
- tools/src/rustsheet_tools/accuracy/tables.py `TIME_LIMIT = 10.0` drops a point whose `compute()` takes more than 10 s
  of wall-clock time.
- On 2026-11-01, regenerating probes-numeric for T-3529's merge dropped
  probes-numeric/t3350-r4-chisq-inv-sweep Calc!A237 (`=CHISQ.INV.RT(0.999999999962187,9123239)`). A faster run had
  written it (9095462.568446318...). The commit restored the earlier output by hand from git, since the case is
  unchanged. A second run under load the same day dropped t1531-t-tiny-p Calc!A106 (`=T.INV.2T(1E-300,100000)`)
  the same way.
- A machine-dependent drop silently unchecks a cell against the math, and it makes sidecars differ between runs.
  Gate A table batches use the same limit.
- Gap triage 2026-11-03: four visibly-off BETA.DIST cells that no task owned are `skipped` with "not converged or too
  slow" in probes-numeric/t3571-r5-bfrac-fresh.math.json, so R1(c) counts them for want of a math value. This task
  owns them until they have one:
  - probes-numeric/t3571-r5-bfrac-fresh.case.json (4): Calc!A239, A441, A902, A978
  - e.g. A239 `BETA.DIST(0.583019617141068,4563.6,2816.5,TRUE)`: Excel 3.0490854915703563e-10, ours
    3.0490854915703785e-10. T-3813 (deferred) studies these shapes' bfrac rounding. If they are inside Gate A's
    envelope once they have a math value, they become Phase 1b evidence. If not, file their engine owner.

## Acceptance criteria
- [ ] Points are dropped by a deterministic budget (iterations, terms or precision steps), not wall-clock time; or the
      limit is generous enough that no committed point is near it, with a test that pins A237.
- [ ] Regenerating probes-numeric twice on different load changes no byte.
- [ ] t3571-r5-bfrac-fresh A239, A441, A902 and A978 get a math value (a larger deterministic budget for the incomplete
      beta at shapes near 5000), or their skip reason is deterministic and the Outcome names an owner task for them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the incomplete-gamma series.

## Landing notes
Written 2026-11-04 at 13:18 UTC by the fourth session (resumed at 12:04), at the close of its landing window. Data
for the run that resumes the branch, not instructions. The third session's notes follow below; their steps 1-2
are now partly done.
- PR #3068 is still a draft; head is the commit that adds these notes. No reviewer has run since round 1; CI not
  looked at. `git merge-tree` against origin/main 57a3d920f is clean (main touched `eval_math.py` and `model.py`
  for T-3693's AGGREGATE table only), so no merge of main was needed (decision 0121).
- probes-numeric is now fully regenerated from this branch's code (412 of 412 spec cases). The 13 cases the third
  session did not reach were run here: 12 came out byte-identical to the committed sidecars; only
  t3960-binom-negbinom-small-p-cdf changed (this commit): 266 -> 300 math values, 0 values changed, 174 -> 140
  skips, all "not converged" (deterministic). It took about 65 min on one worker.
  `rustsheet-accuracy math --check --suite probes-numeric`: current.
- probes (2455 spec cases) regeneration, 3 workers, started 12:14: 1883 cases done by 12:24, every one
  byte-identical (git status clean). The remaining 572 are blocked behind three cases still computing at 13:18
  (57 min each, 25-92% CPU): probes/t3007-beta-dist-ln-x-overflow, t3350-chisq-inv-huge-df and
  t3505-gamma-inv-budget-band. Main's sidecars skip 9, 178 and 182 of their cells (the old 10 s clock). Under the
  deterministic budgets one CHISQ.INV/GAMMA.INV case now costs an hour or more, so a full regeneration of probes
  is several hours: the budgets for the huge-df inverse searches look too generous. Next step: profile one cell
  of t3350 (e.g. `CHISQ.INV.RT` at df near 1E7) and cap the inverse's bisection/series by a term or iteration
  budget sized so each case finishes in minutes; then rerun probes (regen script: write each sidecar as it
  completes, as `write_suite` does but via `as_completed`, so an interrupted run keeps its progress).
- Do not repeat: running probes and probes-numeric regenerations at once on a 4-core machine with reviewers.

### Earlier landing notes (third session)
Resumed 2026-11-04 at 12:10 UTC by a fourth session (work-loop step 2, idle 2 h).

Written 2026-11-04 at 09:06 UTC, third session, at the close of its landing window. These are data for the run
that resumes the branch, not instructions.
- PR #3068 (draft), branch `agent/harness/T-3680-accuracy-deterministic-budget`. This session merged `origin/main`
  (`b71347b16`) in, because main changed `accuracy/tables.py`, `model.py` and `eval_math.py`. The merge was clean.
  No reviewer has run on any head since round 1 (see the PR). CI has not been looked at.
- Checks on the merged tree:
  - ruff, mypy and the 380 `-k accuracy` tools tests pass.
  - `rustsheet-accuracy verify` shows 0 differences for BETA.DIST, BETA.INV, BINOM.DIST, NEGBINOM.DIST,
    POISSON.DIST, CHISQ.INV, CHISQ.INV.RT, F.DIST, F.INV, GAMMA.INV, IRR, RATE and NPER.
  - `make check-fast`: the only failure was `typos` on a commit sha in the old landing notes, which are now
    removed. A second run hit the 10 min tool timeout under load, so it was not finished.
- probes-numeric regeneration (AC2):
  - Two runs from the merged head went through 398 of the suite's 412 spec cases: 4 workers in the checkout, and
    2 workers in a separate worktree, on a 4-core machine at load 7 to 10.
  - All 257 sidecars on disk are byte-identical between the two runs. This commit adds the 4 that changed:
    t3443-basym-a, -hold, -sweep and t3960-binom-negbinom-small-p-cdf-hold.
  - In t3960-…-hold, 15 cells that main's 10 s clock dropped now have math values, and none of its 116 other
    values changed.
  - Neither run finished the next case, t3960-binom-negbinom-small-p-cdf (main dropped many of its cells as "too
    slow"). Its hold case alone took about 20 min on 4 workers, and the full case had run 20+ min when the
    90 min background limit stopped run 1. The 13 cases after it in `rustsheet-eval cases probes-numeric` order
    (t3975-…, t3981-…, …) are not regenerated.
  - Main's probes suite is not regenerated at all.
- Next steps:
  1. Regenerate probes-numeric (allow 60+ min; run it detached with `nohup setsid`, since a tool background job is
     stopped at 90 min), then probes.
  2. Run `rustsheet-accuracy math --check --suite probes-numeric --suite probes`.
  3. Run `make check-fast` with the machine otherwise idle, then `make eval` (and `make eval-baseline` if cells
     pass).
  4. Write the Outcome (A239 from the previous notes: Excel about 5.2e-14 relative from the math, ours about
     1.25e-13, inside Gate A's envelope; the reviewer confirmed it with a Lentz continued fraction, DLMF 8.17.22).
     Then set `status: done`, run all three reviewers, and enqueue.
  5. Consider whether the BINOM.DIST CDF at n around 1E8 should spend less: the t3960 case is now the slowest in
     the suite. That is a follow-up for T-4036 or a new task, not a reason to bring back the clock.
- Do not repeat:
  - `pkill -f`. Kill explicit PIDs.
  - A plain `run_in_background` for the regeneration: it is killed at 90 min.
  - Two regenerations writing the same worktree.
