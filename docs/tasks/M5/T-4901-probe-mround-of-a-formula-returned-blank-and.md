---
id: T-4901
title: "Probe MROUND of a formula-returned blank and LET/LAMBDA omissions in the newer roundings (T-4873 follow-up)"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4873]
---

## Goal
MROUND of a blank that a formula returns, and a LET or LAMBDA variable bound to an omission in CEILING.MATH and
FLOOR.PRECISE, give what Excel gives, decided by a small probe.

## Context
- T-4873 (PR #3932) moved the "written-empty" test of the ToolPak functions onto the argument node
  (`Model::is_empty_argument`, `Model::atp_numbers` in cast.rs). As a result, MROUND now reads a formula-returned blank
  (a VLOOKUP or INDEX landing on an empty cell) as 0, where it used to give `#N/A`. That follows SERIESSUM's evidenced
  `x` (gen-m5/74b183ca6fc1d190 Calc!A16), but no cell probes MROUND itself: numbers.md "An empty argument of a ToolPak
  function is #N/A; an empty significance of the newer roundings is 1" marks it UNVERIFIED.
- The same node test makes `LET(e,,CEILING.MATH(2.675,e))` read `e` as omitted (3). Only QUOTIENT's LET row is
  evidenced (probes/t4873-empty-arg-toolpak-roundings A12).
- Review findings on #3932 (security, design and excel-fidelity, all MINOR).

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) covers `MROUND(VLOOKUP(..blank..),1)`, `MROUND(5,INDEX(<blank>,1))`,
      `MROUND(IF(TRUE,),1)`, plus `LET(e,,CEILING.MATH(2.675,e))`, `LAMBDA(a,b,CEILING.MATH(a,b))(2.675,)`, and the
      same two forms for FLOOR.PRECISE.
- [ ] After the fill every probe cell passes, or the engine is fixed to match; numbers.md drops UNVERIFIED or states
      the corrected rule with its cells.
- [ ] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- QUOTIENT and SERIESSUM written-empty arguments (T-4873, evidenced).
