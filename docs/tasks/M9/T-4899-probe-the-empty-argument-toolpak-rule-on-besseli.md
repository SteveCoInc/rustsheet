---
id: T-4899
title: "Probe the empty-argument ToolPak rule on BESSELI, BESSELY, DEC2OCT, BIN2OCT, HEX2BIN and OCT2HEX"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the empty-argument rule on the six ToolPak engineering functions that share T-4872's code but that
its probe did not call.

## Context
- T-4872 (probes/t4872-atp-empty-argument; engineering.md, "An empty argument of the ToolPak engineering functions")
  routes all of these through `Model::atp_arguments` (functions/engineering/mod.rs): BESSELI, BESSELY, DEC2OCT, BIN2OCT,
  HEX2BIN and OCT2HEX. The rule is: a written-empty required argument is `#N/A`, and an empty `places` is omitted.
  The entry marks these six as unprobed (review of PR #3929).
- Today, for example, `BESSELI(1,)` and `BESSELY(,1)` are `#N/A`, `BIN2OCT(1,)` is "1" and `DEC2OCT(,3)` is `#N/A`.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) writes each argument of the six functions empty, one at a time
      and both at once. The cells are added to probes/t4872-atp-empty-argument or go in a new case.
- [ ] Every cell passes after the fill. If one does not, the rule is fixed in `atp_arguments` and engineering.md says so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The functions T-4872 probed.
