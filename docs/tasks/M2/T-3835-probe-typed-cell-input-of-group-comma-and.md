---
id: T-3835
title: "Probe typed cell input of group-comma and leading-percent texts"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Typing "1,0000", "%1", "%-1", "1," or "0,000" into a cell gives what Excel gives (value and number format).

## Context
T-3811 made the shared reader `parse_formatted_number` read these texts for VALUE and `+0`; typed input shares it and was not probed (coercion.md "Group commas and a leading percent sign", UNPROBED). The `0%`/`0.00%` format a leading percent gets is invented.

## Acceptance criteria
- [ ] A typed-input probe (value and ENTERED numberFormat) for the texts above, filled from excel-graph, and typed input matches it (split the reader from VALUE if they differ).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/typed-group-comma-leading-percent` filled from excel-graph (16 texts). Typed values and
formats already matched the shared reader, so no engine change; added unit test
`typed_group_comma_and_leading_percent_formats` and recorded the finding in coercion.md. Eval delta: none
(new probe case passes 32/32).
