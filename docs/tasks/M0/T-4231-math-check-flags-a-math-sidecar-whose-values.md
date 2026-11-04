---
id: T-4231
title: "math --check flags a math sidecar whose values main's code would no longer write"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math --check` (run by `make check`) fails when a committed math sidecar holds values or skips
that the current generator code would no longer write, not only when a candidate cell is missing.

## Context
- Follow-up of T-3680 (PR #3068), raised by its excel-fidelity review. `check_case`
  (tools/src/rustsheet_tools/accuracy/eval_math.py) checks coverage only: every candidate cell is held or listed as
  skipped, and every entry's `f` matches the cell. It never notices a definition or reader change that alters values.
- Found stale on main by #3068's regeneration, both from code already merged:
  - probes-numeric/t2586-verify-r5-f-edges-hold.math.json: B61, B62, B65, B66, B69 and B70 held tiny values
    (e.g. 7.39e-465) that are 0 since T-3675's subnormal flush in `fitlab/sweep.arithmetic` (#2916); B53 and B54 are
    "no single value".
  - probes-numeric/t1543-inverse-cdf-tiny-p.math.json: A91 and A92 (`BETA.INV(1E-300,0.5,100)`,
    `BETA.INV(1E-200,0.5,100)`) are "no single value" since T-3578 (#2873).
- `--check` must stay cheap (no mpmath; it runs in CI). Candidate designs to weigh: a per-definition version or
  source hash stored in each sidecar entry (or per sidecar) and compared against the current code; storing the parsed
  arguments so a reader change (like the subnormal flush) is caught; a sampled recompute in a nightly job instead.
- A full regeneration of probes takes hours on 4 cores (T-3680's Outcome), so the design should let a change name
  the sidecars it invalidates rather than force a full rerun.

## Acceptance criteria
- [ ] A change to a Gate A definition or to the argument reader that alters a committed math value makes
      `math --check` fail on the affected sidecars, with a test that edits a definition and sees the failure.
- [ ] `math --check` stays mpmath-free and its CI time does not grow by more than 10%.
- [ ] The committed sidecars pass the new check (regenerate only what it flags).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the generator.
