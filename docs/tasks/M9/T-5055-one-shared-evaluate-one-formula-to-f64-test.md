---
id: T-5055
title: "One shared evaluate-one-formula-to-f64 test helper in test/util.rs"
milestone: M9
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine's unit tests share one helper in `crates/rustsheet-engine/src/test/util.rs` that evaluates one formula and
returns its number, and the local copies of it are gone.

## Context
Review follow-up of T-4328 (design reviewer, MINOR): `test/statistical/test_fn_moments.rs` `number`, `test_fn_weibull.rs`
and `test_bug_hunt.rs` each carry their own "set A1, evaluate, read the f64" helper with a different signature
(`rg -n "fn .*\(formula: &str\) -> f64" crates/rustsheet-engine/src/test`). Directive 3: one definition per concept.

## Acceptance criteria
- [ ] One helper in `test/util.rs`; the local copies call it (`scripts/check_duplication.py` does not rise).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Helpers that read text or errors.
