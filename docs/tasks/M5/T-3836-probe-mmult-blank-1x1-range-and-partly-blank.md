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
- Probe probes/mmult-blank-range-vs-error (16 formulas, excel-graph 2026-11-02). In Excel a blank one-cell range
  first argument is `#VALUE!` before the second argument's error, like a bare blank cell. A multi-cell first argument,
  all blank or partly blank, lets the second argument's error through. Without an error such a matrix is `#VALUE!`.
- The engine already matched every cell (16/16), so no code changed. arrays.md "MMULT: a blank first argument" records
  the results with the evidence cells and gains its `Implemented in:` line.
- Eval delta: probes +16 scored cells, all pass; 0 regressions, 0 new known failures.
- Follow-up: T-3859 (sonnet), the text.md wording MINOR from the T-3809 review (PR #2862), which needs that PR's entry on main.
