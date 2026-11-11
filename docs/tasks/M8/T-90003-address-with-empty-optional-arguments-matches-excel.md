---
id: T-90003
title: "ADDRESS with empty optional arguments matches Excel"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: foundational
depends_on: []
---

## Goal
ADDRESS with an optional argument written empty (alone, or produced by IF/CHOOSE/IFERROR/IFNA/LET/LAMBDA) gives Excel's text.

## Context
Owns: probes/address-empty-args (every visibly-off cell).
- Excel reads an empty abs_num as 1 and an empty a1 as TRUE: `ADDRESS(2,3,,)` is `$C$2`, `ADDRESS(2,3,,FALSE)` `R2C3`, `ADDRESS(9,1,,,"Data")` `Data!$A$9`; ours #VALUE!.
- `ADDRESS(2,3,2,)` and `ADDRESS(2,3,2,IF(TRUE,))` are `C$2` (empty a1 is TRUE); ours `R2C[3]` (we read it as FALSE).
- `ADDRESS(2,3,4,TRUE,)` is `C2` (empty sheet_text adds no prefix); ours `!C2`. `ADDRESS(2,3,3,,"S")` is `S!$C2`; ours `S!R[2]C3`.
- An empty value from IF(TRUE,), CHOOSE(2,4,), IFERROR(1/0,), IFNA(NA(),), LET(e,,..) and a LAMBDA's omitted argument behaves the same: `$C$2`; ours #VALUE!. Array forms (A16, A25) likewise.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/address-empty-args matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
