---
id: T-4136
title: "One spelled_like_a1_cell helper in expressions/utils, and the unbound-name quote pass in the parser"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4040]
---

## Goal
One definition of "a name spelled like an A1 cell", and the parse-time quoting of unbound such names living with the
parser, with no behavior change.

## Context
- Carried from T-4040's design review (PR #3144, MINORs).
- `stringify::bound_variables::spelled_like_a1_cell` is `parse_reference_a1(&name.to_ascii_uppercase()).is_some()`,
  which `expressions/parser/operands.rs` also spells out inline (in `bind_bare_name` and one more place, around
  lines 468 and 680). Move the helper to `expressions/utils` next to `parse_reference_a1` and use it in all three.
- `quote_unbound_cell_shaped_names` is `pub(crate)` in `stringify::bound_variables` but only `Parser::parse`
  (`parser/mod.rs`) calls it: move it to the parser side (e.g. `parser/node_walks.rs`), keeping `walk_names` shared.
- `Parser::parse` gates that pass on `formula.contains("_xlpm.")`, which also matches text inside a string literal.
  That is harmless (the pass touches only unbound cell-shaped names). Add a comment saying so.

## Acceptance criteria
- [ ] One `spelled_like_a1_cell` helper; `rg 'parse_reference_a1\(&.*to_ascii_uppercase' crates/rustsheet-engine/src`
      finds only it.
- [ ] No eval or test change (`make eval-check` unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change (T-4099 owns the stored `_xlpm.f1` display).
