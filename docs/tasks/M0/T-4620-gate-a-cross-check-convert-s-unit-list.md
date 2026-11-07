---
id: T-4620
title: "Gate A: cross-check CONVERT's unit list and factors against the engine's table"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4085]
---

## Goal
A check fails when the Gate A CONVERT definition and the engine disagree on which unit strings exist or on a
unit's factor beyond rounding, so the two tables cannot drift apart silently.

## Context
- T-4085 wrote `UNITS` and `_FACTORS` in `tools/src/rustsheet_tools/accuracy/defs/convert.py` by hand, from
  engineering.md "CONVERT unit names" and the exact unit definitions. The engine's table is in `fn_convert`
  (crates/rustsheet-engine/src/functions/engineering/convert.rs). They agree today: every eval cell outside
  `UNITS` is an Excel error, and the factors differ from the engine's doubles by at most 4.5E-16 relative.
- The check must not make the math depend on the engine. One way: a test that runs the engine (through the
  Python bindings or `rustsheet-eval`) on `CONVERT(1,u,base)` for every string in `UNITS` and a sample of
  strings outside it, and compares it with the definition.

## Acceptance criteria
- [ ] A test or gate compares every `UNITS` string, and the unit names outside it, between the definition and
  the engine (valid or `#N/A`, and the factor within 1E-15 relative).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing either table's values (T-2046, T-2047).
