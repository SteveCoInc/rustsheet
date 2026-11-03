---
id: T-3884
title: "ARRAYTOTEXT: a multi-cell range format argument lifts and spills, and a blank first argument gives #VALUE! over an error format"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
ARRAYTOTEXT matches Excel when the format argument is a multi-cell range (the format lifts per element and spills to the range shape, the array argument is read whole), and when the format is an error and the first argument is a blank reference.

## Context
Found by the T-3512 arg-pairs batch (`make eval-show CASE=gen-m5/af89ea54bae02c71.case.json`). Probe first (excel-probe skill), then record in docs/excel-semantics/text.md.
- af89ea54bae02c71: Calc!A31, A34, A91, A94, A151, A154, A187, A190, A253, A256, A301, A304, A355, A358, A403, A406 are `ARRAYTOTEXT(<scalar>,Data!A1:B2)` or `Data!C3:D4` (a range as the format argument): Excel spills one result per format element (anchor cell e.g. `"2"`, `"abc"`, `""`, `"TRUE"`), ours `#VALUE!`; the eval scores the anchor only, so also check the spill shape (see docs/excel-semantics/text.md: A31:B32 is "2","2","2","{2}").
- af89ea54bae02c71: Calc!A529 and A532 are `ARRAYTOTEXT(range,range)` with the same range as format: Excel `"0, FALSE, , 1"` and `", 1.5, x, 0.1"`, ours `#VALUE!`.
- af89ea54bae02c71: Calc!A499 and A502 are `ARRAYTOTEXT(<blank ref>,#DIV/0!)` and `ARRAYTOTEXT(<blank ref>,<#DIV/0! ref>)`: Excel `#VALUE!`, ours `#DIV/0!`. Related: T-2250.

## Acceptance criteria
- [x] The 20 cells above (af89ea54bae02c71 Calc!A31, A34, A91, A94, A151, A154, A187, A190, A253, A256, A301, A304, A355, A358, A403, A406, A529, A532, A499, A502) leave eval/baseline/gen-m5.txt; no new known failure.
- [x] The spill matches the oracle sidecar cell by cell (A31:B32, A34:B35, A529:B530, A532:B533).
- [x] The probed rule is in docs/excel-semantics/text.md.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns has the right type, error and text (decision 0083).

## Out of scope
- Other ARRAYTOTEXT argument kinds (T-2863).

## Outcome
ARRAYTOTEXT is a lifting-table function (array `WholeByCaller`, `format` `Lift`); `fn_arraytotext` renders at most two texts and the driver spills them to the format's shape. A blank first argument is `#VALUE!` before the format is read. Eval: gen-m5/af89ea54bae02c71 now passes all 178 cells (the 20 owned cells leave eval/baseline/gen-m5.txt); no regressions.
