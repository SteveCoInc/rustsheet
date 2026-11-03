---
id: T-3909
title: "TEXTJOIN: probe an array or range ignore_empty with a reference error delimiter"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
We know, from oracle evidence, what TEXTJOIN gives when ignore_empty is an array or range (lifted per element) and the delimiter is a reference to an error cell, and the engine matches it.

## Context
Follow-up of T-3885's excel-fidelity review (MINOR). T-3885 found that a text or error ignore_empty wins over the
error of a reference delimiter (docs/excel-semantics/text.md "TEXTJOIN: ignore_empty is read before a reference
delimiter's error"; probes/t3885-textjoin-error-delimiter-text-ignore-empty). `fn_textjoin` applies that per lifted
element (`lift_call`), e.g. `TEXTJOIN(<#DIV/0! cell>,{TRUE;"x";#N/A},"a")` would spill `#DIV/0!`, `#VALUE!`, `#N/A`.
No cell evidences the lifted case yet. Probe it with the `excel-probe` skill: array-literal and range ignore_empty
mixing booleans, numbers, text, blanks and errors, against a cell, a range and an error-value delimiter. Use
INDEX and ROWS to read each element.

## Acceptance criteria
- [ ] A probe case with excel-graph oracle values covers lifted ignore_empty against reference and value error delimiters.
- [ ] The finding is in docs/excel-semantics/text.md, and any cell the engine misses has an owner task or is fixed here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scalar ignore_empty (T-3885).
