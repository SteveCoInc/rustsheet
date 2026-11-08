---
id: T-5093
title: "Envelope ratchet in the CI eval gate: E lines, drift, the visible lock and the tier lock (SPEC-0037 step 3)"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-5091]
spec: docs/specs/SPEC-0037-usage-tier-gates.md
---

## Goal
The CI eval gate allows bit drift inside decision 0355's envelope and blocks a cell leaving it, a type or error
going wrong, and a tier gate going from met to unmet.

## Context
- SPEC-0037, design section 6.
- Code: crates/rustsheet-eval/src/baseline.rs (`Baseline`, `diff`, `Delta`), visible_parity.rs, main.rs
  (`cmd_check`), and the accuracy check; CLAUDE.md prime directive 2; docs/process/quality-gates.md.
- `cell_envelope` (accuracy/envelope.rs, landed with T-5091) is the only per-cell envelope test: call it, never
  write a second one.

## Acceptance criteria
- [ ] Baselines carry `E` lines. `diff` reports regressions (new `E`), drift (new `F` without `E`) and moves into
  the envelope, and the check prints `drift N` and `into envelope N` per suite.
- [ ] `update-baseline` records drift without `--allow-regressions`, and still refuses new `E` items in
  unchanged cases.
- [ ] The visible-parity lock counts `Outside` and `WrongType` cells.
- [ ] eval/baseline/usage-gates.txt holds each tier's met state, and the check fails on met → unmet.
- [ ] The migration `make eval-baseline` run is committed: tool output only.
- [ ] CLAUDE.md directive 2 and docs/process/quality-gates.md read as SPEC-0037 says.
- [ ] Unit tests for every new `diff` kind, the lock change and the tier lock.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tier gate computation itself (T-5092), beyond calling it.
