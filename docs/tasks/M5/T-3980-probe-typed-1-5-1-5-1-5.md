---
id: T-3980
title: "Probe typed -%1.5, -%-1.5, 1.5$ and -200$, and VALUE of +%1.5 (T-3742 follow-up)"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the shapes T-3742 left unverified: typed cell input of `-%1.5`, `-%-1.5`, `-% 1.5`,
`1.5$`, `1.5 $` and `-200$` (value and format, or text), and VALUE / `+0` of `-%-1.5`, `-% 1.5`, `-%1e2` and
`+%1.5`; the engine matches what the probe shows.

## Context
T-3742 (PR #3000) taught the shared reader `parse_percent_or_currency` a minus before a leading percent sign and
made coercion and criteria refuse a trailing `$` (probes/t3742-value-percent-currency). Typed input shares the
reader, so typed `-%1.5` now becomes -0.015, while `Model::typed_sign_text_is_number` keeps typed `-200$` a number
as before; neither typed shape is probed. docs/excel-semantics/coercion.md lists them as "Unverified
extrapolations". NUMBERVALUE reads `+%1.5` (functions/string_format.rs); VALUE's reading of it is unprobed.
Use the `excel-probe` skill (typed cells need the typed-input probe form of probes/typed-group-comma-leading-percent).

## Acceptance criteria
- [ ] A probe case with oracle values covers every shape above.
- [ ] The engine matches every cell, or a task owns each cell it misses; coercion.md moves the shapes from
      "Unverified extrapolations" to evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Script digits in typed input (T-3891).

## Outcome
Probe `probes/t3980-typed-percent-currency` (excel-graph) filled; 20/20 cells pass. Engine: `-%-1.5` refused, `+%1.5` read,
typed trailing `$` stays text. Rules moved from "Unverified extrapolations" to evidence in coercion.md.
