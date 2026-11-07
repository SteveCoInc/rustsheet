---
id: T-4229
title: "Oracle sweep for how Excel reads a stored xlsx number: the 61 t3658 Hits!F decimals plus fresh short decimals, refilled to fix the rule's points"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Enough excel-graph evidence to derive, and then to check on points never fitted, the rule by which Excel turns a
stored xlsx number (a constant cell's `<v>` text) into a double, so T-4230 can implement it without fitting blind.

## Context
- Gap triage 2026-11-04 (`make eval` on 726980168). numbers.md "Stored numbers in an xlsx are not always parsed to
  the nearest double" (T-1778) is OPEN, and "Decimals reach a double through an x87 multiply by 10^-n" records
  that probes-numeric/t3658-round-scaling Hits!G1:G61 (`=F1` over a stored decimal such as 0.002877) match the
  literal model at 46 of 61 points with the digits as written and 51 with trailing zeros stripped first; they miss
  at G9, G12:G13, G20:G21, G27:G28 and G42:G44 (F9 9.82E-06, F12 6.529E-05, F13 7.856E-05, F20 8.033E-06,
  F21 9.82E-06, F27 7.767E-08, F28 9.282E-08, F42 6.767E-10, F43 7.785E-10, F44 9.312E-10).
- 61 points with 10 misses, plus the 7 T-1778 points on an unmerged branch, are too few to tell candidate rules
  apart and leave nothing for the decision 0188 R4 hold-out and fresh sweep. T-4230 depends on this task.
- The `<v>` text Excel reads is what spec2xlsx writes: XlsxWriter's `{number:.16G}`, rewritten to `repr` when that
  is lossy (`_store_numbers` in tools/src/rustsheet_tools/spec2xlsx.py, T-0289). Record the exact `<v>` text of
  every point (`rustsheet-eval snapshot` or the xlsx part), since the rule may act on that text rather than on
  the decimal the spec holds.
- Read Excel's double with `*2^k-c` companions, as t3658 does (`=F1*2^70-c`), never through the 17-digit transport
  alone.

## Acceptance criteria
- [ ] A new probes-numeric case, oracle-filled (excel-graph), with stored constants and `=Fn` / `=Fn*2^k-c`
      companions over: the 61 Hits!F decimals again (to check fill-to-fill stability, decision 0112); at least 300
      new short decimals m x 10^-n (m < 1,000,000, n = 1..15) where the x87 multiply and the nearest double
      disagree; 60 controls where they agree; the same values written in plain form (`0.00000982`), exponent form
      (`9.82E-06`) and 17-digit form; and negated values. Hold back a fresh slice of at least 30% of the new
      points, chosen by hash before any scoring, in a second case for T-4230's R4.2 guard.
- [ ] numbers.md "Stored numbers in an xlsx are not always parsed to the nearest double" records the counts per
      form (nearest double, literal x87 model as written, trailing zeros stripped), and states which point sets are
      training and which are held out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the xlsx reader (T-4230).
- Number text typed into a cell, VALUE, criteria and NUMBERVALUE (T-4074).
