---
id: T-4900
title: "XLOOKUP/XMATCH empty mode: tell a written-empty argument by its node, not its value"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
XLOOKUP's and XMATCH's `match_mode`/`search_mode` take their default only where the argument is written empty
(or is a LET/LAMBDA variable bound to nothing), not where a value merely evaluates to empty, as for the ToolPak date
functions (T-4870).

## Context
- T-4871 (#3919) added `mode_or_default` (functions/lookup_and_reference/xmatch.rs), which maps a `CalcResult::EmptyArg`
  value to the default. A value can be `EmptyArg` without being written empty: T-4870 found `IFNA(<blank>,..)`
  evaluates to it, where Excel reads the blank (gen-m6/b54448af3fde07ca A297 for EDATE). So
  `XLOOKUP(2,A1:A3,B1:B3,,,IFNA(<blank>,1))` likely takes search_mode 1 in ours where Excel reads 0 (`#VALUE!`).
  Unprobed.
- T-4870 (#3925) put the node-level rule in functions/util/empty_args.rs (`Model::is_written_empty`,
  `Model::written_empty_required`, `written_empty_or`). Depends on it merging.
- lookup.md "XLOOKUP and XMATCH read a written-empty mode as omitted (T-4871)".

## Acceptance criteria
- [ ] A probe (predictions before the fill) holds `XLOOKUP`/`XMATCH` with a mode argument that evaluates to empty
      without being written so (`IFNA(<blank>,..)`, a LET name bound to a blank cell), beside written-empty ones.
- [ ] The engine decides from the node (`is_written_empty`) where the probe says so; probes/t4871-empty-mode-args stays
      31/31.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' empty arguments.
