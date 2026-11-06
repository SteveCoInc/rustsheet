---
id: T-4415
title: "XIRR: probe a literal or one-cell-range boolean or text value against an error dates argument, and with guess -1"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
XIRR's reading of one boolean or text value against an error dates argument is evidenced for every argument form the engine treats alike, not only a cell reference, and the engine matches it.

## Context
T-3816 (PR #3435) made one values cell that is a boolean or non-numeric text against an error dates argument
`#VALUE!` (`XIRR(<TRUE>,<#DIV/0!>)`, probes/t3816-xirr-element-reading D39). A one-cell reference evaluates to its
value before the check, so the engine gives the same `#VALUE!` to literals (`XIRR(TRUE,NA())`, `XIRR("x",1/0)`) and
to a one-cell range `A1:A1` holding text. None of these has an oracle cell (T-3816 fidelity review). With a guess at or
below -1 (`XIRR(<TRUE>,<#DIV/0!>,-1)`) the engine keeps the `#VALUE!` rather than T-3312's `#NUM!`, also unprobed.
financial.md "XIRR reads its values and dates element by element, the date first (T-3816)". Code: `Model::fn_xirr`
(the `size_error` for an error dates argument) in crates/rustsheet-engine/src/functions/financial/mod.rs. Use the
excel-probe skill.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers a literal TRUE, FALSE and "x" value, a one-cell range `A1:A1` holding TRUE and "x", and a cell TRUE with guess -1 and -2, each against a literal error dates argument and an error dates cell.
- [ ] The engine matches every probed cell; financial.md's T-3816 entry records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Values ranges of more than one cell (gen-m9/67a4c03afd63f988 A241 already evidences them).
