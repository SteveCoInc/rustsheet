---
id: T-4598
title: "Split rustsheet-eval runner.rs below the size limit before the next pivot change"
milestone: M16
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
crates/rustsheet-eval/src/runner.rs has room for the coming pivot work: spec-case preparation (`prepare_spec`, `spec_formulas` and the expectation building) moves to its own module, runner/prepare.rs, and runner.rs is well under the 1000-line limit.

## Context
After T-4126, runner.rs is 977 lines (design review of T-4126). T-4124 already moved scoring to runner/scoring.rs. T-4131 (xlsx-case pivots) and T-4152 (the adapter rendering pivots) both add to case preparation. The split is a pure move with no behaviour change.

## Acceptance criteria
- [ ] runner.rs under 800 lines; no behaviour change (`make eval-check` identical); tests unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Behaviour changes.
