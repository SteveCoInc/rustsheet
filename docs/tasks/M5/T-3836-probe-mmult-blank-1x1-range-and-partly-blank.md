---
id: T-3836
title: "Probe MMULT blank 1x1 range and partly-blank multi-cell first argument beside an error"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Establish with the excel-probe skill what `MMULT(<blank>, <error>)` gives when the blank first argument is a 1x1 range (`Data!H1:H1`) or a multi-cell range with blanks (`Data!H1:H2`), and make the engine match.

## Context
T-3729 made a single blank first argument `#VALUE!` before the second argument's error (mmult.rs; arrays.md "MMULT: a blank first argument"). Oracle evidence covers only bare single-cell references (gen-m5/842a0aebcb17b83d A463, A466). The engine's check also fires for a 1x1 range, while a multi-cell partly-blank first argument still propagates the second argument's error. Both are unprobed. Also add the `Implemented in:` line to that arrays.md entry.

## Acceptance criteria
- [x] Probe cases for both forms have Excel values; arrays.md records the results and an `Implemented in:` line.
- [x] The engine matches them.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions.

## Outcome
- Probe probes/t3836-mmult-blank-range-first-arg (excel-graph, 2026-11-02): 19 cells (18 questions + the Data
  error cell). In Excel, a blank one-cell range (`Data!H1:H1`) as the first argument is `#VALUE!` before the second
  argument's error. A multi-cell first argument, all blank (`H1:H2`, `H1:I1`) or partly blank (`A1:A2`, `A1:B1`),
  lets the second argument's error through, even when the dimensions do not match.
- The engine already matched every cell (`fn_mmult`'s 1x1 `ArrayNode::Empty` check), so the code does not change.
  arrays.md records the rule and gains the `Implemented in:` line.
- Eval delta: +1 probe case, 19 cells, all passing; 0 regressions.
