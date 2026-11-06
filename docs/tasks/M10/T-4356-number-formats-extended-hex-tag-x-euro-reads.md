---
id: T-4356
title: "Number formats: extended-hex tag [x-euro] reads as byte 2 but Excel prints ASCII digits"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(123,"[$-x-euro]0")` is "123" as in Excel; the extended-hex reading of a non-hex, non-BCP-47 tag no longer picks Arabic-Indic digits for it.

## Context
T-4107 probed `[$-x-euro]0` on 123: Excel prints "123" while `locale_tag::extended_hex` gives byte 2 ("١٢٣"). Probe it again with more extended-hex tags (excel-probe) to find the rule (docs/excel-semantics/number-formats.md "Extended-hex tags stay ASCII past byte 31"); code: `locale_tag::numerals`.

## Acceptance criteria
- [ ] A probe case holds `[$-x-euro]0` and its siblings, all matching Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Rule found by probe: a tag opening `x-` (either case) prints ASCII digits at every extended-hex numeral byte; the same values without the prefix keep their numerals. `locale_tag::numerals` now skips the extended-hex reading for `x-` tags. Probes: t4356-extended-hex-xeuro, t4356-private-use-prefix, t4356-private-use-value. Documented in docs/excel-semantics/number-formats.md. Eval delta: 4 new probe cases (t4356-*) added to eval/baseline/probes.txt, every cell matching Excel; known failures unchanged, 0 regressions. Also corrects the earlier "ASCII past byte 31" reading: bytes 32+ of extended-hex tags take their digits (that ASCII result was the x- prefix).
