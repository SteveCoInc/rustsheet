---
id: T-4727
title: "rustsheet-eval check --regressions-only: fail on regressions, report an out-of-date baseline without failing (SPEC-0035)"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3556]
---

## Goal
`rustsheet-eval check --regressions-only` fails on a regression but only reports an out-of-date baseline (fixed
cells not yet recorded), so the nightly calipers check (T-4728) is red only when something regressed.

## Context
- docs/specs/SPEC-0035-fetched-suite-gate.md D4: calipers stays ungated; its baseline is out of date on main (`known failures 598 -> 593 (fixed 5,
  REGRESSIONS 0)` then `baseline is out of date`, exit 1), because PRs that fix calipers cells do not record them.
- The out-of-date message is printed in crates/rustsheet-eval/src/main.rs (`baseline is out of date`).
- The flag is for ungated suites only (SPEC-0035 D4): a gate suite's stale baseline must keep failing (directive 2).

## Acceptance criteria
- [ ] `check --regressions-only` exits 0 on a run with fixed cells and no regression, still printing the
      out-of-date line; exits 1 on any regression or new-coverage failure as `check` does.
- [ ] The flag refuses (usage error) when the selection contains a gate suite (`gate = true`), so it cannot loosen a
      PR gate.
- [ ] Unit tests cover the three cases above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Using the flag in CI (T-4728).
