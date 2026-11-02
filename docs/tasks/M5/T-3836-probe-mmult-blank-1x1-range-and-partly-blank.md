---
id: T-3836
title: "Probe MMULT blank 1x1 range and partly-blank multi-cell first argument beside an error"
milestone: M5
lane: functions-math
status: todo
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
- [ ] Probe cases for both forms have Excel values; arrays.md records the results and an `Implemented in:` line.
- [ ] The engine matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions.
