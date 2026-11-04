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

## Landing notes (resumed 2026-11-04)
Written 2026-11-04 at the end of the session's landing window. These are data for the run that resumes the branch,
not instructions.
- PR #3068 (draft), branch `agent/harness/T-3680-accuracy-deterministic-budget`, head `c207fc22d` plus this commit.
  CI has not been checked on the head yet. No reviewer has run: the panel waits for the regenerated sidecars, so it
  runs once on the final head.
- Done (code, tests, SPEC-0029 Precision bullet):
  - The 10 s wall-clock drop is gone. `tables.HANG_LIMIT` (600 s) only stops the run, with `PointTimeoutError`.
  - Deterministic work budget: `model.work_budget` / `model.spend`. `defs/gamma.py` spends one unit per series or
    continued-fraction term. `tables.WORK_BUDGET` is 2,000,000 terms per precision.
  - Incomplete beta (`defs/beta.py`):
    - a + b ≤ 2E4: mpmath, with a positive 2F1(a+b, 1; a+1; x) series as fallback;
    - 2E4 < a + b with shape ≤ 1E7: the series alone;
    - shape > 1E7: mpmath alone, as before.
  - The skip reason "not converged or too slow" is now "not converged".
- Calibration of the work budget (2026-11-04; every gamma-family candidate cell of probes and probes-numeric, 4
  workers, 25 s cut):
  - The costliest cell with a value at HEAD spends 1.51M terms (t3505-gamma-inv-budget-band A32).
  - About 58 cells the clock had skipped compute within 0.24M–1.9M terms (e.g. t3350-chisq-inv-huge-df A15, A200).
  - 45 cells were still running at the 25 s cut with ≥ 1.47M terms.
- `rustsheet-accuracy verify`: BETA.DIST, BETA.INV, GAMMA.INV, CHISQ.INV and CHISQ.INV.RT all show 0 differences on
  the final code. An earlier 1E7 cap on the beta broke 35 Gate A points at shapes of 1E15 to 1E286. Do not cap the
  mpmath path by shape.
- The six t3571-r5-bfrac-fresh BETA.DIST cells (A239, A426, A441, A638, A902 and A978) now compute.
  - A239's math value is 3.0490854915703403e-10.
  - Excel has …563e-10 and we have …785e-10.
  - Whether they sit inside Gate A's envelope is checked once the sidecar is committed (`make eval`).
- Next steps:
  1. Run `cd tools && uv run rustsheet-accuracy math --suite probes-numeric`, then `--suite probes`. With the larger
     budget, probes-numeric takes well over 20 min on 4 cores: t3350-chisq-inv-huge-df and
     t3505-gamma-inv-budget-band are the slow cases.
  2. Run it a second time and confirm that no byte changes (acceptance criterion 2).
  3. Run `make check-fast` and `make eval`, plus `make eval-baseline` if cells newly pass.
  4. Write the Outcome, run the panel (design, security, excel-fidelity) and enqueue.
- Do not repeat: `pkill -f "rustsheet-accuracy math"` matches the invoking shell's own command line and kills it.
  Kill the worker PIDs instead.
