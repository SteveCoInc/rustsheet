---
id: T-4902
title: "let.rs reuses Model::is_empty_argument for its own empty-or-omitted check (T-4873 follow-up)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4873]
---

## Goal
The LET/LAMBDA argument check in crates/rustsheet-engine/src/functions/logical/let.rs uses `Model::is_empty_argument`
instead of spelling out the same test, so the test has one definition (directive 3).

## Context
- T-4873 (PR #3932) added `Model::is_empty_argument` (let.rs): `matches!(node, Node::EmptyArgKind) ||
  self.is_omitted_variable(node)`. It also moved the complex functions' check onto it. The check near let.rs:283
  (`if matches!(node, Node::EmptyArgKind) || self.is_omitted_variable(node)`) is the same test, still written inline
  (design review MINOR on #3932).
- `mmult.rs` tests only the literal `EmptyArgKind`, which is a different test. Leave it.

## Acceptance criteria
- [ ] let.rs calls `self.is_empty_argument(node)` at that check; no behaviour change (engine lib tests and
      `make check-fast` pass, eval delta 0).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other empty-argument checks that test something else (mmult.rs).
