---
id: T-3975
title: "CONVERT with a binary and a decimal prefix: Gibyte to zbyte (t3861-convert-prefix-sweep A52)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
CONVERT between a binary-prefixed and a decimal-prefixed information unit matches Excel to the last bit.

## Context
T-3861 applies both prefixes as one factor, the nearest double to their combined power of ten times
the binary prefixes' exact ratio (docs/excel-semantics/engineering.md "CONVERT constants and evaluation
order"). One fresh-sweep cell mixes the two kinds and misses by 1 ulp:
probes-numeric/t3861-convert-prefix-sweep Calc!A52 `CONVERT(4.73E-10,"Gibyte","zbyte")` is
5.0787988275200007E+20 in Excel, 5.07879882752E+20 here (× 1E21 × 2^30). `* 2^30 * (1/1E-21)` (the engine's
form before T-3861) gives Excel's value. The sweep's other three mixed cells pass. One cell is not enough to
choose a rule: probe the mixed pairs (every binary prefix against a spread of decimal prefixes, both
directions, bit and byte, a few values) on excel-graph first.
Code: the prefix factor at the end of `convert_of` in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [ ] A probe of mixed binary/decimal prefix pairs is oracle-filled, and the rule it shows is in engineering.md.
- [ ] A52 and the probe's cells pass, and no CONVERT cell of any suite changes from pass to fail.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Light-year and parsec factor digits (T-2047).
