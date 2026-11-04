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

## Landing notes (resumed 2026-11-04 third session)
Written 2026-11-04, second session, at the close of its landing window. These are data for the run that resumes the
branch, not instructions.
- PR #3068 (draft), branch `agent/harness/T-3680-accuracy-deterministic-budget`. The code head is `ca8616215`. This
  commit adds the notes and follow-up T-4036.
- Reviews:
  - Round 1, on `5f97fe9ed`: security APPROVE; design APPROVE with MINORs.
    - The test MINOR is fixed in `ae4ba79cd`. The rest is filed as T-4036.
  - Excel fidelity: REQUEST_CHANGES, two MAJORs.
    1. t3443-basym-sweep A150 and A154 (BETA.INV at b about 6E4) lost their math value. Fixed in `26d42cc9d` and
       `ca8616215`:
       - `beta._positive_2f1` sums 2F1(a+b, 1; a+1; x) by hand with `spend()`.
       - `_series` takes the cheaper side by `_terms`. It keeps a complement only while the tail is at most
         1 - 1E-6.
       - `_tail_series` goes to `mpmath.hyp2f1` at once when `_terms` is over `SERIES_MAXTERMS`. Without that,
         Gate A BETA.INV/s1 A176 breaks.
       - `mpmath.hyp2f1` past z of about 0.75 took 329 s in a hypercomb at I_0.832(62750.652, 383).
       - Results: A150 gives 0.0061942141340931975056697909694474336368428487 (main's value). A154 matches main too.
         `rustsheet-accuracy verify BETA.DIST` and `BETA.INV` show 0 differences, and a unit test pins A150.
    2. The task is unfinished: probes not regenerated, no two-run byte check, no Outcome. Still open, see the next
       steps.
- Sidecars: the probes-numeric sidecars committed in `5f97fe9ed` came from the code before `26d42cc9d`, so they must
  be regenerated.
  - Some of their changes are drift already on main, not T-3680:
    - t2586-verify-r5-f-edges-hold B53/B54 are now "no single value" and B61..B70 are 0.0. `fitlab.sweep.arithmetic`
      flushes a subnormal operand to 0, as Excel does.
    - t1543-inverse-cdf-tiny-p A91/A92 are now "no single value". Main's `quantile` also gives NoValueError there
      (UNDERFLOW_SOURCE).
  - A239 is inside Gate A's envelope: Excel is about 5.2e-14 relative from the math, ours about 1.25e-13. The fidelity
    reviewer confirmed A239 independently with a Lentz continued fraction (DLMF 8.17.22). Record both in the Outcome.
- When the window closed, a regeneration of probes and probes-numeric (2 workers each, from `ca8616215`) was stopped
  unfinished. Its first three probes sidecars (t3007, t3350, t3352) are committed as written. Every sidecar still needs
  the full regeneration of step 1.
- Next steps:
  1. Run `cd tools && .venv/bin/rustsheet-accuracy math --suite probes-numeric`, then `--suite probes`.
     - Call the venv binary: two concurrent `uv run` calls failed with "Failed to spawn".
     - probes-numeric takes about 33 min on 4 workers.
  2. Run both again under a different load and confirm no byte changes (AC2).
  3. Run `make check-fast` and `make eval`, plus `make eval-baseline` if cells newly pass.
  4. Write the Outcome. Set `status: done`.
  5. Re-run all three reviewers on the final head, then enqueue.
- Do not repeat:
  - Choosing the side by the switch point alone: A150's root solve then spends 1.9M terms.
  - Rejecting a complement whose tail is above 1/2: the upper tail then takes the 6E4-term side.
  - `pkill -f "rustsheet-accuracy math"`: it kills the invoking shell. Kill the python worker PIDs instead.
